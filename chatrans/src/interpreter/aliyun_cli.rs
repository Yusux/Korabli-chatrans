/**
 * Reference: https://help.aliyun.com/zh/machine-translation/developer-reference/signature-mechanism
 */
use md5::{Md5, Digest};
use base64::prelude::*;
use sha1::Sha1;
use hmac::{Hmac, Mac};
use url::Url;
use uuid::Uuid;

const SIGNATURE_METHOD: &str = "HMAC-SHA1";
const SIGNATURE_VERSION: &str = "2019-01-02";

pub struct AliyunCli {
    access_key_id: String,
    access_key_secret: String,
}

impl AliyunCli {
    pub fn new(access_key_id: String, access_key_secret: String) -> AliyunCli {
        AliyunCli {
            access_key_id,
            access_key_secret,
        }
    }

    /// Calculate the MD5 hash of a string and encode it in base64
    fn md5_base64(s: &str) -> String {
        // create a new hasher to calculate the MD5 hash
        let mut hasher = Md5::new();
        hasher.update(s.as_bytes());
        // get the hash result
        let md5_result = hasher.finalize();

        // encode the hash result in base64
        BASE64_STANDARD.encode(&md5_result)
    }

    /// Calculate the HMAC-SHA1 hash of a string and encode it in base64
    fn hmacsha1_base64(data: &str, key: &str) -> String {
        // create a new HMAC-SHA1 hasher
        let mut hasher: Hmac<Sha1> = Mac::new_from_slice(key.as_bytes()).unwrap();
        // update the hasher with the data
        hasher.update(data.as_bytes());
        // get the hash result
        let result = hasher.finalize().into_bytes();

        // encode the hash result in base64
        BASE64_STANDARD.encode(&result)
    }

    // Get time with format: "E, dd MMM yyyy HH:mm:ss z"
    fn get_time() -> String {
        chrono::Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string()
    }

    /// Get the access url
    pub async fn send_post(
        &self,
        url: String,
        body: String,
    ) -> String {
        let url = Url::parse(&url).unwrap();

        // set the request parameters
        let method = "POST";
        let accept = "application/json";
        let content_type = "application/json;chrset=utf-8";
        let path = url.path();
        let date = Self::get_time();
        let host = url.host_str().unwrap();
        let uuid = Uuid::new_v4().to_string();

        // apply md5 and base64 to the body
        let body_md5 = Self::md5_base64(&body);
        let string_to_sign = format!(
            "{}\n{}\n{}\n{}\n{}\nx-acs-signature-method:{}\nx-acs-signature-nonce:{}\nx-acs-version:{}\n{}",
            method,
            accept,
            body_md5,
            content_type,
            date,
            SIGNATURE_METHOD,
            uuid,
            SIGNATURE_VERSION,
            path
        );
        // apply hmac-sha1 and base64 to the string to sign
        let signature = Self::hmacsha1_base64(&string_to_sign, &self.access_key_secret);
        // set the authorization header
        let authorization = format!(
            "acs {}:{}", self.access_key_id, signature
        );

        // connect to the server
        let client = reqwest::Client::new();
        let response = client.post(url.clone())
            .header("Accept", accept)
            .header("Content-Type", content_type)
            .header("Content-MD5", body_md5)
            .header("Date", date)
            .header("Host", host)
            .header("Authorization", authorization)
            .header("x-acs-signature-nonce", uuid)
            .header("x-acs-signature-method", SIGNATURE_METHOD)
            .header("x-acs-version", SIGNATURE_VERSION)
            .body(body)
            .send()
            .await
            .unwrap();

        response.text().await.unwrap()
    }
}
