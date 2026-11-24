use reqwest::{Client, Response};

pub struct HttpClient {
    client: Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    pub async fn post<T, B>(&self, url: &str, body: &B) -> Result<T, reqwest::Error>
    where
        T: serde::de::DeserializeOwned,
        B: serde::Serialize + ?Sized,
    {
        let response = self.post_raw(url, body).await?.error_for_status()?;

        Ok(response.json::<T>().await?)
    }

    pub async fn post_raw<B>(&self, url: &str, body: &B) -> Result<Response, reqwest::Error>
    where
        B: serde::Serialize + ?Sized,
    {
        self.client.post(url).json(body).send().await
    }
}
