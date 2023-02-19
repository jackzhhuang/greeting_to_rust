
use crate::DataSet::PersonData::Person; 

#[derive(Debug)]
pub struct HouseInfo {
    pub address: String,
    pub no: u32, 
    pub someone: Person,
}