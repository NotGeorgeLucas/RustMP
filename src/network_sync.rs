pub trait NetworkSync  {
    fn get_owner(&self) -> i32;
    fn set_owner(&mut self, owner_id: i32);
}