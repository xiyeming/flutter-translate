pub struct TranslationEngine {
    initialized: bool,
}

impl TranslationEngine {
    pub fn new() -> Self {
        Self { initialized: false }
    }

    pub async fn init(&mut self) -> Result<(), crate::ffi::error::TranslateError> {
        crate::translate::init_router().await;
        self.initialized = true;
        Ok(())
    }

    pub fn is_initialized(&self) -> bool {
        self.initialized
    }
}
