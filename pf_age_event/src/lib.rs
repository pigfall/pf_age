#[derive(Debug)]
pub enum Event{
    SystemEvent(SystemEvent),
    InputEvent(InputEvent),
}

#[derive(Debug)]
pub struct InputEvent{

}

#[derive(Debug)]
pub enum SystemEvent{
    AndroidNativeWindowCreated,
    AndroidNativeWindowDestoryed,
    AndroidNativeInputQueueCreated,
    AndroidNativeInputQueueDestroyed,
}
