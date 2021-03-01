use std::collections::HashMap;

//responsible for starting and stoping services
// data is passed between service via message passing channels
// data is passed between applications via memory file, pipe or socket

#[derive(Debug)]
pub enum ServiceError {
    //error when, called to start a service that is already active
    already_active,
    //errpr when, called to stop a service that is not active
    not_active,
    //windows api call failed,
    windows_get_last_error,
}

#[derive(Debug, PartialEq, Eq, Hash)]
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
    fn join(&mut self);
}

#[derive(Debug)]
pub enum InputType {
    user,
    telemetry,
    image,
}

// input method type could be shared memory, pipe ,
pub trait Input {
    fn input_type(&self) -> InputType;
    // implement packet size

    // serialize input to a portable format (i.e json/xml or jpeg/png)

    // serialize to compressed format for miminal on disk size
}

//every services has a process trait, to stop and start the specific services
// all input services
pub struct Services {
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
    pub fn add_service(
        &mut self,
        service_type: ServiceType,
        mut process: Box<dyn Process>,
    ) -> Result<(), ServiceError> {
        process.start()?;
        self.services.insert(service_type, process);

        Ok(())
    }

    //stops a running service else nothing
    pub fn stop_service(&mut self, service_type: ServiceType) -> Result<(), ServiceError> {
        if self.services.contains_key(&service_type) {
            let mut p = self.services.remove(&service_type).unwrap();
            p.stop()?;

            p.join();
            println!("service stopped {:?}", service_type);
        }

        Ok(())
    }

    //stop all services
    pub fn stop_all_services(&mut self) -> Result<(), ServiceError> {
        for (k, v) in self.services.iter_mut() {
            v.stop()?;
            v.join();
            println!("service stopped {:?}", k);
        }

        Ok(())
    }

    //block until all services has joined
    //pub fn block_wait() {}
}
