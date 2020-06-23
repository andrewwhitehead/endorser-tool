#[macro_use]
extern crate serde_derive;

use indy_vdr::common::did::DidValue;
use indy_vdr::common::error::{input_err, VdrResult};
use indy_vdr::pool::PreparedRequest;

use indy_utils::base58;
use indy_utils::did::generate_did;
use indy_utils::keys::{EncodedVerKey, SignKey};

use web_view::{Content, WVResult, WebView};

const PRELOAD: &'static str = "<link href=js/app.js rel=preload as=script>";
const MARKER: &'static str = "src=js/app.js>";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct EndorserInfo {
    did: String,
    verkey: String,
    verkey_long: String,
}

#[derive(Default)]
struct UserData {
    endorser: EndorserInfo,
    key: Option<SignKey>,
}

fn main() {
    // FIXME should be done in a build script
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
        .size(480, 520)
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
                UpdateSeed { seed } => match create_did(seed.as_bytes()) {
                    Ok((key, endorser)) => {
                        data.key.replace(key);
                        data.endorser = endorser.clone();
                        SetEndorser {
                            endorser,
                            error: None,
                        }
                    }
                    Err(err) => {
                        data.key.take();
                        data.endorser = EndorserInfo::default();
                        SetEndorser {
                            endorser: data.endorser.clone(),
                            error: Some(err.to_string()),
                        }
                    }
                },
                UpdateDid { did } => match base58::decode(&did) {
                    Ok(did_bytes) => {
                        if did_bytes.len() == 16 {
                            let verkey = EncodedVerKey::new(&data.endorser.verkey_long, None, None);
                            data.endorser.verkey = verkey.abbreviated_for_did(&did).unwrap();
                            data.endorser.did = did;
                            SetEndorser {
                                endorser: data.endorser.clone(),
                                error: None,
                            }
                        } else {
                            SetEndorser {
                                endorser: data.endorser.clone(),
                                error: Some("DID value must be 16 bytes in length".to_string()),
                            }
                        }
                    }
                    Err(_) => SetEndorser {
                        endorser: data.endorser.clone(),
                        error: Some("Invalid base58 format for DID value".to_string()),
                    },
                },
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

fn create_did<S: AsRef<[u8]>>(seed: S) -> VdrResult<(SignKey, EndorserInfo)> {
    let (did, sk, vk) = generate_did(Some(seed.as_ref()))
        .map_err(|err| input_err(format!("Error generating DID: {}", err)))?;
    let verkey_long = vk.as_base58().unwrap();
    let verkey = verkey_long.abbreviated_for_did(&did).unwrap();
    Ok((
        sk,
        EndorserInfo {
            did: did.to_string(),
            verkey,
            verkey_long: verkey_long.to_string(),
        },
    ))
}

fn sign_transaction(data: &UserData, txn: String) -> VdrResult<String> {
    let mut req = PreparedRequest::from_request_json(txn)?;
    let sigin = req.get_signature_input()?;
    let sig = data.key.as_ref().unwrap().sign(sigin.as_bytes()).unwrap();
    req.set_multi_signature(&DidValue(data.endorser.did.clone()), &sig)?;
    Ok(serde_json::to_string(&req.req_json).unwrap())
}

#[derive(Debug, Serialize, Deserialize)]
struct Task {
    name: String,
    done: bool,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
    Init,
    Log { text: String },
    UpdateSeed { seed: String },
    UpdateDid { did: String },
    SignTransaction { txn: String },
}

#[derive(Debug, Serialize)]
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
