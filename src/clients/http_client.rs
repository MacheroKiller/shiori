use reqwest::Client;

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
        let res = self
            .client
            .post(url)
            .json(body)
            .send()
            .await?
            .error_for_status()?;
        Ok(res.json::<T>().await?)
    }
}
