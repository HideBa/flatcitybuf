// automatically generated by the FlatBuffers compiler, do not modify

// @generated

extern crate flatbuffers;

pub enum ExtensionOffset {}
#[derive(Copy, Clone, PartialEq)]

pub struct Extension<'a> {
    pub _tab: flatbuffers::Table<'a>,
}

impl<'a> flatbuffers::Follow<'a> for Extension<'a> {
    type Inner = Extension<'a>;
    #[inline]
    unsafe fn follow(buf: &'a [u8], loc: usize) -> Self::Inner {
        Self {
            _tab: flatbuffers::Table::new(buf, loc),
        }
    }
}

impl<'a> Extension<'a> {
    pub const VT_NAME: flatbuffers::VOffsetT = 4;
    pub const VT_DESCRIPTION: flatbuffers::VOffsetT = 6;
    pub const VT_URL: flatbuffers::VOffsetT = 8;
    pub const VT_VERSION: flatbuffers::VOffsetT = 10;
    pub const VT_VERSION_CITYJSON: flatbuffers::VOffsetT = 12;
    pub const VT_EXTRA_ATTRIBUTES: flatbuffers::VOffsetT = 14;
    pub const VT_EXTRA_CITY_OBJECTS: flatbuffers::VOffsetT = 16;
    pub const VT_EXTRA_ROOT_PROPERTIES: flatbuffers::VOffsetT = 18;
    pub const VT_EXTRA_SEMANTIC_SURFACES: flatbuffers::VOffsetT = 20;

    #[inline]
    pub unsafe fn init_from_table(table: flatbuffers::Table<'a>) -> Self {
        Extension { _tab: table }
    }
    #[allow(unused_mut)]
    pub fn create<'bldr: 'args, 'args: 'mut_bldr, 'mut_bldr, A: flatbuffers::Allocator + 'bldr>(
        _fbb: &'mut_bldr mut flatbuffers::FlatBufferBuilder<'bldr, A>,
        args: &'args ExtensionArgs<'args>,
    ) -> flatbuffers::WIPOffset<Extension<'bldr>> {
        let mut builder = ExtensionBuilder::new(_fbb);
        if let Some(x) = args.extra_semantic_surfaces {
            builder.add_extra_semantic_surfaces(x);
        }
        if let Some(x) = args.extra_root_properties {
            builder.add_extra_root_properties(x);
        }
        if let Some(x) = args.extra_city_objects {
            builder.add_extra_city_objects(x);
        }
        if let Some(x) = args.extra_attributes {
            builder.add_extra_attributes(x);
        }
        if let Some(x) = args.version_cityjson {
            builder.add_version_cityjson(x);
        }
        if let Some(x) = args.version {
            builder.add_version(x);
        }
        if let Some(x) = args.url {
            builder.add_url(x);
        }
        if let Some(x) = args.description {
            builder.add_description(x);
        }
        if let Some(x) = args.name {
            builder.add_name(x);
        }
        builder.finish()
    }

    #[inline]
    pub fn name(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(Extension::VT_NAME, None)
        }
    }
    #[inline]
    pub fn description(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(Extension::VT_DESCRIPTION, None)
        }
    }
    #[inline]
    pub fn url(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(Extension::VT_URL, None)
        }
    }
    #[inline]
    pub fn version(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(Extension::VT_VERSION, None)
        }
    }
    #[inline]
    pub fn version_cityjson(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(Extension::VT_VERSION_CITYJSON, None)
        }
    }
    #[inline]
    pub fn extra_attributes(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(Extension::VT_EXTRA_ATTRIBUTES, None)
        }
    }
    #[inline]
    pub fn extra_city_objects(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab
                .get::<flatbuffers::ForwardsUOffset<&str>>(Extension::VT_EXTRA_CITY_OBJECTS, None)
        }
    }
    #[inline]
    pub fn extra_root_properties(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(
                Extension::VT_EXTRA_ROOT_PROPERTIES,
                None,
            )
        }
    }
    #[inline]
    pub fn extra_semantic_surfaces(&self) -> Option<&'a str> {
        // Safety:
        // Created from valid Table for this object
        // which contains a valid value in this slot
        unsafe {
            self._tab.get::<flatbuffers::ForwardsUOffset<&str>>(
                Extension::VT_EXTRA_SEMANTIC_SURFACES,
                None,
            )
        }
    }
}

impl flatbuffers::Verifiable for Extension<'_> {
    #[inline]
    fn run_verifier(
        v: &mut flatbuffers::Verifier,
        pos: usize,
    ) -> Result<(), flatbuffers::InvalidFlatbuffer> {
        v.visit_table(pos)?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>("name", Self::VT_NAME, false)?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>(
                "description",
                Self::VT_DESCRIPTION,
                false,
            )?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>("url", Self::VT_URL, false)?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>("version", Self::VT_VERSION, false)?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>(
                "version_cityjson",
                Self::VT_VERSION_CITYJSON,
                false,
            )?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>(
                "extra_attributes",
                Self::VT_EXTRA_ATTRIBUTES,
                false,
            )?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>(
                "extra_city_objects",
                Self::VT_EXTRA_CITY_OBJECTS,
                false,
            )?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>(
                "extra_root_properties",
                Self::VT_EXTRA_ROOT_PROPERTIES,
                false,
            )?
            .visit_field::<flatbuffers::ForwardsUOffset<&str>>(
                "extra_semantic_surfaces",
                Self::VT_EXTRA_SEMANTIC_SURFACES,
                false,
            )?
            .finish();
        Ok(())
    }
}
pub struct ExtensionArgs<'a> {
    pub name: Option<flatbuffers::WIPOffset<&'a str>>,
    pub description: Option<flatbuffers::WIPOffset<&'a str>>,
    pub url: Option<flatbuffers::WIPOffset<&'a str>>,
    pub version: Option<flatbuffers::WIPOffset<&'a str>>,
    pub version_cityjson: Option<flatbuffers::WIPOffset<&'a str>>,
    pub extra_attributes: Option<flatbuffers::WIPOffset<&'a str>>,
    pub extra_city_objects: Option<flatbuffers::WIPOffset<&'a str>>,
    pub extra_root_properties: Option<flatbuffers::WIPOffset<&'a str>>,
    pub extra_semantic_surfaces: Option<flatbuffers::WIPOffset<&'a str>>,
}
impl Default for ExtensionArgs<'_> {
    #[inline]
    fn default() -> Self {
        ExtensionArgs {
            name: None,
            description: None,
            url: None,
            version: None,
            version_cityjson: None,
            extra_attributes: None,
            extra_city_objects: None,
            extra_root_properties: None,
            extra_semantic_surfaces: None,
        }
    }
}

pub struct ExtensionBuilder<'a: 'b, 'b, A: flatbuffers::Allocator + 'a> {
    fbb_: &'b mut flatbuffers::FlatBufferBuilder<'a, A>,
    start_: flatbuffers::WIPOffset<flatbuffers::TableUnfinishedWIPOffset>,
}
impl<'a: 'b, 'b, A: flatbuffers::Allocator + 'a> ExtensionBuilder<'a, 'b, A> {
    #[inline]
    pub fn add_name(&mut self, name: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(Extension::VT_NAME, name);
    }
    #[inline]
    pub fn add_description(&mut self, description: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(Extension::VT_DESCRIPTION, description);
    }
    #[inline]
    pub fn add_url(&mut self, url: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(Extension::VT_URL, url);
    }
    #[inline]
    pub fn add_version(&mut self, version: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_
            .push_slot_always::<flatbuffers::WIPOffset<_>>(Extension::VT_VERSION, version);
    }
    #[inline]
    pub fn add_version_cityjson(&mut self, version_cityjson: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            Extension::VT_VERSION_CITYJSON,
            version_cityjson,
        );
    }
    #[inline]
    pub fn add_extra_attributes(&mut self, extra_attributes: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            Extension::VT_EXTRA_ATTRIBUTES,
            extra_attributes,
        );
    }
    #[inline]
    pub fn add_extra_city_objects(&mut self, extra_city_objects: flatbuffers::WIPOffset<&'b str>) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            Extension::VT_EXTRA_CITY_OBJECTS,
            extra_city_objects,
        );
    }
    #[inline]
    pub fn add_extra_root_properties(
        &mut self,
        extra_root_properties: flatbuffers::WIPOffset<&'b str>,
    ) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            Extension::VT_EXTRA_ROOT_PROPERTIES,
            extra_root_properties,
        );
    }
    #[inline]
    pub fn add_extra_semantic_surfaces(
        &mut self,
        extra_semantic_surfaces: flatbuffers::WIPOffset<&'b str>,
    ) {
        self.fbb_.push_slot_always::<flatbuffers::WIPOffset<_>>(
            Extension::VT_EXTRA_SEMANTIC_SURFACES,
            extra_semantic_surfaces,
        );
    }
    #[inline]
    pub fn new(_fbb: &'b mut flatbuffers::FlatBufferBuilder<'a, A>) -> ExtensionBuilder<'a, 'b, A> {
        let start = _fbb.start_table();
        ExtensionBuilder {
            fbb_: _fbb,
            start_: start,
        }
    }
    #[inline]
    pub fn finish(self) -> flatbuffers::WIPOffset<Extension<'a>> {
        let o = self.fbb_.end_table(self.start_);
        flatbuffers::WIPOffset::new(o.value())
    }
}

impl core::fmt::Debug for Extension<'_> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let mut ds = f.debug_struct("Extension");
        ds.field("name", &self.name());
        ds.field("description", &self.description());
        ds.field("url", &self.url());
        ds.field("version", &self.version());
        ds.field("version_cityjson", &self.version_cityjson());
        ds.field("extra_attributes", &self.extra_attributes());
        ds.field("extra_city_objects", &self.extra_city_objects());
        ds.field("extra_root_properties", &self.extra_root_properties());
        ds.field("extra_semantic_surfaces", &self.extra_semantic_surfaces());
        ds.finish()
    }
}
