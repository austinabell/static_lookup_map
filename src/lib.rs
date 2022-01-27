use near_sdk::{collections::LookupMap, near_bindgen};
use std::cell::RefCell;
use std::thread_local;

thread_local! {
    static LOOKUP_MAP: RefCell<LookupMap<String, u8>> = {
        let mut m = LookupMap::new(b"c");

        // These steps will be run on every function call, so be careful what is done here.
        // If you need to just initialize once, perhaps an init function is the best way to go?
        near_sdk::env::log_str("inside thread local init");
        m.insert(&"a".to_string(), &1);
        m.insert(&"b".to_string(), &2);
        m.into()
    };
}

#[near_bindgen]
pub struct SomeContract;

#[near_bindgen]
impl SomeContract {
    pub fn get_val(s: &String) -> Option<u8> {
        LOOKUP_MAP.with(|m| m.borrow().get(s))
    }

    pub fn set_val(s: String, v: u8) -> Option<u8> {
        LOOKUP_MAP.with(|m| m.borrow_mut().insert(&s, &v))
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use tokio::fs;
    use workspaces::prelude::*;

    #[tokio::test]
    async fn workspaces_test() -> anyhow::Result<()> {
        let wasm = fs::read("res/const_lm.wasm").await?;

        let worker = workspaces::sandbox();

        let contract = worker.dev_deploy(wasm).await?;

        let res = contract
            .call(&worker, "get_val")
            .args_json(serde_json::json! {{"s": "a"}})?
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert_eq!(res.json::<u8>()?, 1);

        let res = contract
            .call(&worker, "set_val")
            .args_json(serde_json::json! {{"s": "e", "v": 8}})?
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert!(res.json::<Option<u8>>()?.is_none());

        let res = contract
            .call(&worker, "get_val")
            .args_json(serde_json::json! {{"s": "e"}})?
            .gas(300_000_000_000_000)
            .transact()
            .await?;
        assert_eq!(res.json::<Option<u8>>()?, Some(8));
        Ok(())
    }
}
