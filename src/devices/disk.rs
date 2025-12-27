pub type Disk = Vec<DiskSection>;
#[derive(Debug,Clone)]
pub struct DiskSection {
    pub section_type: DiskSectionType,
    pub data: Vec<i16>,
    pub id: i16,
}
#[derive(Debug, PartialEq,Clone)]
pub enum DiskSectionType {
    Entrypoint,
    Libary,
    Code,
    Loader,
    Data,
}