use crate::core::config::Config;
use crate::core::engine::Engine as DaggerEngine;
use crate::core::graphql_client::DefaultGraphQLClient;
use futures::FutureExt;
use std::panic::AssertUnwindSafe;
use std::sync::Arc;

use crate::errors::ConnectError;
use crate::gen::Query;
use crate::querybuilder::query;

pub type DaggerConn = Query;

pub async fn connect<F, Fut>(dagger: F) -> Result<(), ConnectError>
where
    F: FnOnce(DaggerConn) -> Fut + 'static,
    Fut: futures::Future<Output = eyre::Result<()>> + 'static,
{
    let cfg = Config::builder().build();

    connect_opts(cfg, dagger).await
}

pub async fn connect_opts<F, Fut>(cfg: Config, dagger: F) -> Result<(), ConnectError>
where
    F: FnOnce(DaggerConn) -> Fut + 'static,
    Fut: futures::Future<Output = eyre::Result<()>> + 'static,
{
    let (conn, proc) = DaggerEngine::new()
        .start(&cfg)
        .await
        .map_err(ConnectError::FailedToConnect)?;

    let proc = proc.map(Arc::new);

    let client = Query {
        proc: proc.clone(),
        selection: query(),
        graphql_client: Arc::new(DefaultGraphQLClient::new(&conn, &cfg)),
    };

    // XXX: we have to use catch_unwind here because the dagger client functions can panic
    // in particular this happens when the engine is shutdown during some queries.
    // These functions should probably be made faillible instead of panicking ?
    let res = AssertUnwindSafe(dagger(client))
        .catch_unwind()
        .await
        .map_err(|_| ConnectError::FailedToConnect(eyre::eyre!("Panic in dagger client")))
        .map(|res| res.map_err(ConnectError::DaggerContext));

    if let Some(proc) = &proc {
        let shutdown_res = proc
            .shutdown()
            .await
            .map_err(ConnectError::FailedToShutdown);
        if res.is_ok() {
            shutdown_res?;
        }
    }

    res?
}

// Conn will automatically close on drop of proc

#[cfg(test)]
mod test {
    use super::connect;

    #[tokio::test]
    async fn test_connect() -> eyre::Result<()> {
        tracing_subscriber::fmt::init();

        connect(|client| async move {
            client
                .container()
                .from("alpine:latest")
                .with_exec(vec!["echo", "1"])
                .sync()
                .await?;

            Ok(())
        })
        .await?;

        Ok(())
    }
}
