pub use toktor_impl::actor_handler;

#[macro_export]
macro_rules! toktor_send {
    ($actorname:ident,$message:ident) => {
        $actorname.sender.send($message)
    };
}

#[macro_export]
macro_rules! toktor_new {
    ($actorhand:ident, $($x:expr),* ) => {
        {
            $actorhand::new($($x),*)
        }
    };
    ($actorhand:ident) => {
        {
            $actorhand::new()
        }
    };
}
