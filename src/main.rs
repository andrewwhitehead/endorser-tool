#[macro_use]
extern crate serde_derive;

use indy_vdr::common::did::DidValue;
use indy_vdr::common::error::VdrResult;
use indy_vdr::pool::PreparedRequest;

use indy_utils::keys::SignKey;
use indy_utils::ursa::keys::PrivateKey;
use indy_utils::ursa::signatures::{ed25519::Ed25519Sha512, SignatureScheme};

use web_view::{Content, WVResult, WebView};

const PRELOAD: &'static str = "<link href=js/app.js rel=preload as=script>";
const MARKER: &'static str = "src=js/app.js>";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EndorserInfo {
    did: String,
    verkey: String,
}

#[derive(Default)]
struct UserData {
    endorser: EndorserInfo,
    key: Option<SignKey>,
}

fn main() {
    // FIXME should be done in a build script.
    let src_html = include_str!("../web/build/index.html");
    let js = include_str!("../web/build/js/app.js");

    let pos = src_html.find(MARKER).unwrap();
    let mut html = src_html[..pos].to_string().replace(PRELOAD, "");
    html.push('>');
    html.push_str(js);
    html.push_str(&src_html[(pos + MARKER.len())..]);

    let webview = web_view::builder()
        .title("Endorser Tool")
        .content(Content::Html(html))
        .size(480, 480)
        .resizable(false)
        .debug(true)
        .user_data(UserData::default())
        .invoke_handler(|webview, arg| {
            use Cmd::*;
            use Update::*;

            let data = webview.user_data_mut();

            let update = match serde_json::from_str(arg).unwrap() {
                Init => NoUpdate,
                Log { text } => {
                    println!("{}", text);
                    NoUpdate
                }
                UpdateSeed { seed } => {
                    let key = SignKey::from_seed(seed.as_bytes()).unwrap();
                    let verkey = key.public_key().unwrap();
                    let verkey_bytes = verkey.key_bytes().unwrap();
                    let did = bs58::encode(&verkey_bytes[..16]).into_string();
                    let verkey = verkey.as_base58().unwrap().to_string();
                    let endorser = EndorserInfo { did, verkey };
                    data.endorser = endorser.clone();
                    data.key.replace(key);
                    SetEndorser {
                        endorser,
                        error: None,
                    }
                }
                UpdateDid { did } => {
                    data.endorser.did = did;
                    NoUpdate
                }
                SignTransaction { txn } => match sign_transaction(&data, txn) {
                    Ok(txn) => SetSignedOutput { txn, error: None },
                    Err(err) => SetSignedOutput {
                        txn: "".to_string(),
                        error: Some(err.to_string()),
                    },
                },
            };
            // webview.set_title(..)?;

            send_update(webview, &update)
        })
        .build()
        .unwrap();

    //webview.set_color((156, 39, 176));

    let _res = webview.run().unwrap();
}

fn send_update(webview: &mut WebView<UserData>, update: &Update) -> WVResult {
    let upd = format!("app.fromRust({})", serde_json::to_string(update).unwrap());
    webview.eval(&upd)
}

fn sign_transaction(data: &UserData, txn: String) -> VdrResult<String> {
    let mut req = PreparedRequest::from_request_json(txn)?;
    let sigin = req.get_signature_input()?;
    let pk = PrivateKey(data.key.as_ref().unwrap().key_bytes().unwrap());
    let signer = Ed25519Sha512::new();
    let sig = signer.sign(sigin.as_bytes(), &pk).unwrap();
    req.set_multi_signature(&DidValue(data.endorser.did.clone()), &sig)?;
    Ok(serde_json::to_string(&req.req_json).unwrap())
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    name: String,
    done: bool,
}

#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
    Init,
    Log { text: String },
    UpdateSeed { seed: String },
    UpdateDid { did: String },
    SignTransaction { txn: String },
}

#[derive(Serialize)]
#[serde(tag = "update", rename_all = "camelCase")]
pub enum Update {
    NoUpdate,
    SetEndorser {
        endorser: EndorserInfo,
        error: Option<String>,
    },
    SetSignedOutput {
        txn: String,
        error: Option<String>,
    },
}
