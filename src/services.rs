use std::collections::HashMap;

//responsible for starting and stoping services

pub enum ServiceError {
    //error when, called to start a service that is already active
    already_active,
    //errpr when, called to stop a service that is not active
    not_active,
    //windows api call failed,
    windows_get_last_error,
}

pub enum ServiceType {
    user_input,         // could be keyboard or controller
    telemetry_input,    // could be different games, for now only ets2_telemetry
    image_input,        // screenshot
    synchronize_inputs, // use to synchronize and groupify the inputs
    feedback,           // use display the user and telemetry outputs and image
}
pub trait Process {
    fn start(&mut self) -> Result<(), ServiceError>;
    fn stop(&mut self) -> Result<(), ServiceError>;
    fn join(self);
}

pub enum InputType {
    user,
    telemetry,
    image,
}

pub trait Input {
    fn identity(&self) -> &str; // identify the specific

    // implement packet size

    // serialize input to a portable format (i.e json/xml or jpeg/png)

    // serialize to compressed format for miminal on disk size
}

//every services has a process trait, to stop and start the specific services
// all input services
struct Services {
    // hashmap to keep all  service objects (contain thread handle)
    services: HashMap<ServiceType, Box<dyn Process>>,
}

impl Services {
    // create services objects to manage all services
    pub fn new() -> Services {
        Services {
            services: HashMap::<ServiceType, Box<dyn Process>>::new(),
        }
    }
    //service needs to impliment process thread
    pub fn add_service(&mut self, service_type: ServiceType, process: Box<dyn Process>) {}

    //stops a running service else nothing
    pub fn stop_service(&mut self, service_type: ServiceType) {}

    //stop all services
    pub fn stop_all_services(&mut self) {}

    //block until all services has joined
    pub fn block_wait() {}
}
