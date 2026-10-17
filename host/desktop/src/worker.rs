/// Always wake the UI after the worker has unwound, including on resource-drop
/// panics. The failed guest/store is consumed and is never reused after a panic.
pub fn run<T>(
    work: impl FnOnce() -> anyhow::Result<T>,
    request_exit: impl FnOnce(),
) -> anyhow::Result<T> {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(work));
    request_exit();
    match result {
        Ok(result) => result,
        Err(payload) => {
            let message = payload
                .downcast_ref::<String>()
                .map(String::as_str)
                .or_else(|| payload.downcast_ref::<&str>().copied())
                .unwrap_or("unknown panic payload");
            Err(anyhow::anyhow!("emulator thread panicked: {message}"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run;
    use std::cell::Cell;

    #[test]
    fn requests_exit_on_success_and_error() {
        let exits = Cell::new(0);
        assert_eq!(run(|| Ok(42), || exits.set(exits.get() + 1)).unwrap(), 42);
        let result = run::<()>(
            || Err(anyhow::anyhow!("guest failed")),
            || exits.set(exits.get() + 1),
        );
        assert_eq!(result.unwrap_err().to_string(), "guest failed");
        assert_eq!(exits.get(), 2);
    }

    #[test]
    fn requests_exit_after_execution_panic_and_unwinding() {
        struct Resource<'a>(&'a Cell<bool>);
        impl Drop for Resource<'_> {
            fn drop(&mut self) {
                self.0.set(true);
            }
        }
        let dropped = Cell::new(false);
        let notified = Cell::new(false);
        let result = run::<()>(
            || {
                let _resource = Resource(&dropped);
                panic!("host call failed");
            },
            || {
                assert!(dropped.get(), "UI must stay alive through resource cleanup");
                notified.set(true);
            },
        );
        assert!(notified.get());
        assert!(result.unwrap_err().to_string().contains("host call failed"));
    }

    #[test]
    fn requests_exit_when_resource_cleanup_panics() {
        struct Resource;
        impl Drop for Resource {
            fn drop(&mut self) {
                panic!("GPU cleanup failed");
            }
        }
        let notified = Cell::new(false);
        let result = run(
            || {
                drop(Resource);
                Ok(())
            },
            || notified.set(true),
        );
        assert!(notified.get());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("GPU cleanup failed"));
    }
}
