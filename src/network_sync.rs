pub trait NetworkSync  {
    fn get_owner(&self) -> i32;
    fn set_owner(&mut self, owner_id: i32);

    fn get_object_id(&self) -> i32;
    fn set_object_id(&mut self, object_id: i32);
}