const RP_STATE_PREFIX: &str = "state_rp_id_";

pub(super) const RP_COOKIE_PATH: &str = "/resources/providers";

pub(super) fn get_rp_state_cookie_name(rp_name: &String) -> String {
    format!("{RP_STATE_PREFIX}{rp_name}")
}
