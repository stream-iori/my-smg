use crate::write_timeout::WriteTimeoutListener;
use axum::Router;
use std::{error::Error, io, time::Duration};
use tokio::{
    sync::oneshot,
    task::JoinHandle,
    time::{Instant, timeout_at},
};

// 健康检查清理和请求排空，共用这 5 秒预算。
// 不是每个步骤分别等待 5 秒。
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(5);

// 等待一个停机信号。
//
// 成功时返回信号名称，供日志记录。
// 失败表示注册或监听信号出现问题。
async fn wait_for_shutdown() -> io::Result<&'static str> {
    // 在 macOS、Linux 等 Unix 系统上，编译这一段。
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        // 注册 SIGTERM 监听器。
        //
        // kill -TERM <PID> 通常会发送这个信号。
        // 注册失败时，? 提前返回 io::Error。
        let mut terminate = signal(SignalKind::terminate())?;

        // 同时等待两个 Future。
        // 任意一个先完成，就执行它的分支；
        // 另一个等待分支随之被取消。
        tokio::select! {
            // 终端按 Ctrl+C 通常产生 SIGINT。
            result = tokio::signal::ctrl_c() => {
                // ctrl_c() 返回 io::Result<()>。
                // 监听失败就返回错误；成功则返回名称。
                result?;
                Ok("ctrl_c")
            }

            // 等待 SIGTERM。
            signal = terminate.recv() => {
                match signal {
                    // 收到一次信号。
                    Some(()) => Ok("sigterm"),

                    // 信号流关闭，不等于正常收到停机信号。
                    None => Err(io::Error::other(
                        "termination signal stream closed",
                    )),
                }
            }
        }
    }

    // 非 Unix 系统只编译这个版本。
    // 两个 cfg 分支不会同时存在于最终编译结果中。
    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c().await?;
        Ok("ctrl_c")
    }
}

// 构造统一的停机超时错误。
//
// &'static str 用来接收源码中的固定字符串，
// 例如 "health_task"、"request_drain"。
fn timeout_error(phase: &'static str) -> io::Error {
    io::Error::new(
        io::ErrorKind::TimedOut,
        format!("shutdown deadline exceeded during {phase}"),
    )
}

// 取得服务器监听器、路由和健康检查任务的所有权，
// 统一负责它们的运行与停机。
//
// mut health_task：后面需要通过 &mut 等待句柄。
// Box<dyn Error>：允许返回 io::Error、JoinError 等不同错误。
pub(crate) async fn run(
    listener: WriteTimeoutListener,
    app: Router,
    mut health_task: JoinHandle<()>,
) -> Result<(), Box<dyn Error>> {
    // 创建只能发送一次消息的通道。
    //
    // shutdown_tx：当前管理流程持有，负责发通知。
    // shutdown_rx：交给服务器，负责接收通知。
    //
    // 消息类型是 ()，因为只需要表达“开始停机”，
    // 不需要携带其他数据。
    let (shutdown_tx, shutdown_rx) = oneshot::channel::<()>();

    // 将服务器放到独立 Tokio 任务中运行。
    //
    // async move 把 listener、app、shutdown_rx
    // 的所有权移动进这个任务。
    //
    // 这样即使管理流程正在等待健康检查清理，
    // 服务器仍能独立推进已有连接和停机流程。
    let mut server_task = tokio::spawn(async move {
        axum::serve(listener, app)
            .with_graceful_shutdown(async move {
                // 这个 Future 完成时，Axum 开始优雅停机。
                //
                // 收到 ()：正常触发停机。
                // 发送端被销毁：await 返回错误，也触发停机。
                //
                // 两种情况都应该停止服务，因此这里忽略结果。
                let _ = shutdown_rx.await;
            })
            // 等待服务器完全结束。
            //
            // 收到通知后，Axum 停止接收新连接，
            // 并等待已有连接完成优雅关闭。
            .await
    });

    // 暂存信号监听错误。
    //
    // 即使监听出错，也先通知服务器停机并清理后台任务，
    // 最后再把错误返回给 main。
    let mut signal_error = None;

    // 服务运行期间等待两个事件：
    // 1. 服务器任务自己结束。
    // 2. 收到停机信号，或者监听信号失败。
    let server_result = tokio::select! {
        // &mut 表示借用 JoinHandle 等待，
        // 不把句柄移动进这个分支 Future。
        //
        // 如果另一分支先完成，后面还能继续等待 server_task。
        result = &mut server_task => {
            // 已经拿到服务器任务的结果，先保存。
            // 后面还要清理健康检查任务。
            Some(result)
        }

        signal = wait_for_shutdown() => {
            match signal {
                Ok(signal) => {
                    tracing::info!(
                        event = "shutdown_requested",
                        signal = signal,
                    );
                }

                Err(error) => {
                    tracing::error!(
                        event = "shutdown_signal_failed",
                        error = %error,
                    );

                    // 保留错误，先做资源清理。
                    signal_error = Some(error);
                }
            }

            // 通知服务器开始优雅停机。
            //
            // send 消费发送端：oneshot 只能发送一次。
            // 若接收端已经消失，说明服务器已不再等待通知，
            // 不需要因为这次发送失败而中断清理。
            let _ = shutdown_tx.send(());

            // 此时尚未拿到服务器结束结果。
            None
        }
    };

    // select! 结束后，正式进入清理阶段。
    //
    // 使用绝对截止时间，而不是每一步重新计算 5 秒，
    // 避免多个清理步骤把总耗时不断延长。
    let deadline = Instant::now() + SHUTDOWN_TIMEOUT;

    // 请求取消健康检查任务。
    //
    // abort() 只是发出取消请求，不表示任务已经结束。
    // 必须继续 await，才能确认取消已完成。
    health_task.abort();

    let health_error = match timeout_at(deadline, &mut health_task).await {
        // 外层 Ok：没有超时。
        // 内层 Ok：健康检查任务正常结束。
        Ok(Ok(())) => None,

        // 健康检查被我们主动取消，这是预期结果。
        Ok(Err(error)) if error.is_cancelled() => None,

        // 任务已经结束，但出现其他 JoinError，
        // 例如任务内部发生 panic。
        //
        // 保存错误，继续给服务器排空请求的机会。
        Ok(Err(error)) => {
            tracing::error!(
                event = "health_task_failed",
                error = %error,
            );
            Some(error)
        }

        // 连健康检查清理都没能在预算内完成。
        // 已没有时间继续等待服务器排空。
        Err(_) => {
            server_task.abort();

            tracing::error!(event = "shutdown_deadline_exceeded", phase = "health_task",);

            return Err(timeout_error("health_task").into());
        }
    };

    // 到这里，健康检查任务已经结束。
    // 若任务 panic，前面还会有 health_task_failed 日志。
    tracing::info!(event = "health_task_stopped");

    if let Some(result) = server_result {
        // select! 已经取到了服务器结果，
        // 不可以再重复 await 一个已经取过结果的 JoinHandle。
        //
        // result 的类型有两层：
        //
        // Result<
        //     io::Result<()>,
        //     JoinError,
        // >
        //
        // 第一个 ?：处理服务器任务的 JoinError。
        // 第二个 ?：处理 Axum 返回的 io::Error。
        result??;
    } else {
        // 服务器正在优雅停机。
        // 使用同一个 deadline，等待已有请求结束。
        match timeout_at(deadline, &mut server_task).await {
            Ok(result) => {
                // 没有超过期限，再分别检查两层结果。
                result??;
            }

            Err(_) => {
                // 排空超时，取消服务器外层任务。
                server_task.abort();

                // 等待外层任务确认取消。
                // 此时取消产生的 JoinError 是预期结果。
                let _ = server_task.await;

                tracing::error!(
                    event = "shutdown_deadline_exceeded",
                    phase = "request_drain",
                );

                // 返回 main 后，main 结束，Tokio runtime 关闭。
                // 仍运行的异步连接任务由 runtime 关闭流程终止。
                //
                // 仅 abort 外层服务器任务，不能据此认定所有
                // 已派生的连接任务也都已经停止。
                return Err(timeout_error("request_drain").into());
            }
        }
    }

    // 服务器与健康检查均已处理完，
    // 再报告此前保存的错误。
    if let Some(error) = signal_error {
        return Err(error.into());
    }

    if let Some(error) = health_error {
        return Err(error.into());
    }

    // 只有在清理、排空成功且没有保存的错误时，
    // 才记录正常停机完成。
    tracing::info!(event = "shutdown_finished");

    Ok(())
}
