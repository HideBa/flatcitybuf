use crate::deserializer::to_cj_feature;
use crate::error::Error;
use crate::fb::*;
use cjseq::CityJSONFeature;

pub struct FcbBuffer {
    pub header_buf: Vec<u8>,
    pub features_buf: Vec<u8>,
}

impl FcbBuffer {
    pub fn header(&self) -> Header {
        unsafe { size_prefixed_root_as_header_unchecked(&self.header_buf) }
    }

    pub fn feature(&self) -> CityFeature {
        unsafe { size_prefixed_root_as_city_feature_unchecked(&self.features_buf) }
    }

    // TODO: think well if needed
    pub fn cj_feature(&self) -> Result<CityJSONFeature, Error> {
        let fcb_feature = self.feature();
        let root_attr_schema = self.header().columns();
        to_cj_feature(fcb_feature, root_attr_schema)
    }
}
