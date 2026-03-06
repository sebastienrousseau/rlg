#![allow(missing_docs)]
#[cfg(test)]
mod tests {
    use rlg::tracing::RlgSubscriber;
    use tracing::{error, info, warn};
    use tracing_core::dispatcher::{self, Dispatch};

    #[test]
    fn test_tracing_integration() {
        let subscriber = RlgSubscriber::new();
        let dispatch = Dispatch::new(subscriber);
        
        dispatcher::with_default(&dispatch, || {
            info!(target: "my_comp", key = "val", "This is an info message from tracing");
            warn!("This is a warning");
            error!("This is an error");
        });
    }
}
