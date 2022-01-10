#[cfg(test)]
mod tests {
    use std::future::Future;

    #[cfg(feature = "async-std-runtime")]
    fn run_test<F: Future>(future: F) -> F::Output {
        async_std::task::block_on(future)
    }
    #[cfg(feature = "tokio-runtime")]
    fn run_test<F: Future>(future: F) -> F::Output {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(future)
    }

    use async_mongodb_session::*;
    use async_session::{Session, SessionStore};
    use lazy_static::lazy_static;
    use mongodb::{options::ClientOptions, Client};
    use rand::Rng;
    use std::env;

    lazy_static! {
        static ref HOST: String = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
        static ref PORT: String = env::var("PORT").unwrap_or_else(|_| "27017".to_string());
        static ref CONNECTION_STRING: String =
            format!("mongodb://{}:{}/", HOST.as_str(), PORT.as_str());
    }

    async fn from_client() -> async_session::Result {
        let client_options = match ClientOptions::parse(CONNECTION_STRING.as_str()).await {
            Ok(c) => c,
            Err(e) => panic!("Client Options Failed: {}", e),
        };

        let client = match Client::with_options(client_options) {
            Ok(c) => c,
            Err(e) => panic!("Client Creation Failed: {}", e),
        };

        let store = MongodbSessionStore::from_client(client, "db_name", "collection");
        let mut rng = rand::thread_rng();
        let n2: u16 = rng.gen();
        let key = format!("key-{}", n2);
        let value = format!("value-{}", n2);
        let mut session = Session::new();
        session.insert(&key, &value)?;

        let cookie_value = store.store_session(session).await?.unwrap();
        let session = store.load_session(cookie_value).await?.unwrap();
        assert_eq!(&session.get::<String>(&key).unwrap(), &value);

        Ok(())
    }

    async fn new() -> async_session::Result {
        let store = MongodbSessionStore::new(&CONNECTION_STRING, "db_name", "collection").await?;

        let mut rng = rand::thread_rng();
        let n2: u16 = rng.gen();
        let key = format!("key-{}", n2);
        let value = format!("value-{}", n2);
        let mut session = Session::new();
        session.insert(&key, &value)?;

        let cookie_value = store.store_session(session).await?.unwrap();
        let session = store.load_session(cookie_value).await?.unwrap();
        assert_eq!(&session.get::<String>(&key).unwrap(), &value);

        Ok(())
    }

    async fn with_expire() -> async_session::Result {
        let store = MongodbSessionStore::new(&CONNECTION_STRING, "db_name", "collection").await?;

        store.initialize().await?;

        let mut rng = rand::thread_rng();
        let n2: u16 = rng.gen();
        let key = format!("key-{}", n2);
        let value = format!("value-{}", n2);
        let mut session = Session::new();
        session.expire_in(std::time::Duration::from_secs(5));
        session.insert(&key, &value)?;

        let cookie_value = store.store_session(session).await?.unwrap();
        let session = store.load_session(cookie_value).await?.unwrap();
        assert_eq!(&session.get::<String>(&key).unwrap(), &value);

        Ok(())
    }

    async fn check_expired() -> async_session::Result {
        use async_std::task;
        use std::time::Duration;
        let store = MongodbSessionStore::new(&CONNECTION_STRING, "db_name", "collection").await?;

        store.initialize().await?;

        let mut rng = rand::thread_rng();
        let n2: u16 = rng.gen();
        let key = format!("key-{}", n2);
        let value = format!("value-{}", n2);
        let mut session = Session::new();
        session.expire_in(Duration::from_secs(1));
        session.insert(&key, &value)?;

        let cookie_value = store.store_session(session).await?.unwrap();

        task::sleep(Duration::from_secs(1)).await;
        let session_to_recover = store.load_session(cookie_value).await?;

        assert!(&session_to_recover.is_none());

        Ok(())
    }

    #[test]
    fn test_from_client() -> async_session::Result {
        run_test(from_client())
    }

    #[test]
    fn test_new() -> async_session::Result {
        run_test(new())
    }

    #[test]
    fn test_with_expire() -> async_session::Result {
        run_test(with_expire())
    }

    #[test]
    fn test_check_expired() -> async_session::Result {
        run_test(check_expired())
    }
}
