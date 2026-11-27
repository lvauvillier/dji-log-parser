use binrw::binread;
use serde::Serialize;
#[cfg(target_arch = "wasm32")]
use tsify_next::Tsify;

#[binread]
#[derive(Serialize, Debug)]
#[serde(rename_all = "camelCase")]
#[br(little)]
#[cfg_attr(target_arch = "wasm32", derive(Tsify))]
pub struct AppGPS {
    /// degrees
    pub latitude: f64,
    /// degrees
    pub longitude: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use binrw::io::Cursor;
    use binrw::BinRead;

    #[test]
    fn latitude_comes_first() {
        let data: Vec<u8> = (0..40)
            .step_by(2)
            .map(|i| {
                u8::from_str_radix(&"46879afbbedd4940eb7e3b763d962c400000d842"[i..i + 2], 16)
                    .unwrap()
            })
            .collect();

        let gps = AppGPS::read_le(&mut Cursor::new(data)).unwrap();

        assert_eq!((gps.latitude.round(), gps.longitude.round()), (52.0, 14.0));
    }
}
