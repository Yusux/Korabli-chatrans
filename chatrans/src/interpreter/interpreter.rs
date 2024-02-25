use serde_json;
use tracing::warn;

use crate::interpreter::aliyun_cli::AliyunCli;

const SERVER_URL: &str = "http://mt.cn-hangzhou.aliyuncs.com/api/translate/web/ecommerce";
const FORMAT_TYPE: &str = "text";
const SOURCE_LANGUAGE: &str = "auto";
const SCENE: &str = "social";

pub enum Language {
    ZH,
    EN,
}

impl From<String> for Language {
    fn from(s: String) -> Self {
        match s.as_str() {
            "zh" => Language::ZH,
            "en" => Language::EN,
            _ => {
                warn!("Invalid language: {}, default to `zh`", s);
                Language::ZH
            }
        }
    }
}

impl Language {
    pub fn to_string(&self) -> String {
        match self {
            Language::ZH => "zh".to_string(),
            Language::EN => "en".to_string(),
        }
    }
}

pub struct Interpreter {
    language: Language,
    aliyun_cli: Option<AliyunCli>,
}

impl Interpreter {
    pub fn new(
        language: String,
        access_key_id: Option<String>,
        access_key_secret: Option<String>,
    ) -> Interpreter {
        let language = Language::from(language);
        let aliyun_cli = match (access_key_id, access_key_secret) {
            (Some(id), Some(secret)) => Some(AliyunCli::new(id, secret)),
            _ => None,
        };

        Interpreter {
            language,
            aliyun_cli,
        }
    }

    pub async fn translate(&self, text: String) -> String {
        if self.aliyun_cli.is_none() {
            return text;
        }

        // Format the post body
        let post_body = format!(
            "{{\n\"FormatType\": \"{}\",\n\"SourceLanguage\": \"{}\",\n\"TargetLanguage\": \"{}\",\n\"SourceText\": \"{}\",\n\"Scene\": \"{}\"\n}}",
            FORMAT_TYPE,
            SOURCE_LANGUAGE,
            self.language.to_string(),
            text,
            SCENE
        );

        // Send the post request
        let response = self.aliyun_cli.as_ref().unwrap().send_post(
            SERVER_URL.to_string(),
            post_body,
        ).await;

        // Parse the response
        let response: serde_json::Value = serde_json::from_str(&response).unwrap();
        // Check if the response is valid
        if response["Code"] != "200" || response["Data"]["Translated"].is_null() {
            warn!("Failed to translate the text: {:?}", response);
            return text;
        }

        // Return the translated text
        response["Data"]["Translated"].as_str().unwrap().to_string()
    }
}
