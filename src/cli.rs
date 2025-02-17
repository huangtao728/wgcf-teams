use argh::FromArgs;

#[derive(FromArgs)]
/// Generate WireGuard profile for WARP Teams
pub struct Args {
    /// read private key from stdin instead of generating a new one
    #[argh(switch)]
    pub prompt: bool,

    /// device name to register with
    #[argh(option, short = 'n', default = "String::from(\"wgcf-teams-device\")")]
    pub device_name: String,
}
