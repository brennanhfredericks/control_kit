use crate::telemetry::EventGame;
use std::collections::HashMap;
use std::fmt;

//responsible for starting and stoping services
// data is passed between service via message passing channels
// data is passed between applications via memory file, pipe or socket

#[derive(Debug)]
pub enum ServiceError {
    //error when, called to start a service that is already active
    AlreadyActive,
    //errpr when, called to stop a service that is not active
    NotActive,
    //windows api call failed,
    WindowsGetLastError(i32),
    // transmitter (for data passing between thread) has not been set for service
    TransmitterNotSet,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum ServiceType {
    UserInput,          // could be keyboard or controller
    TelemetryInput,     // could be different games, for now only ets2_telemetry
    ScreenCaptureInput, // screenshot
    SynchronizeInputs,  // use to synchronize and groupify the inputs
                        // use display the user and telemetry outputs and image
}
pub trait Process {
    fn start(&mut self) -> Result<(), ServiceError>;
    fn stop(&mut self) -> Result<(), ServiceError>;
    fn join(&mut self);
}

#[derive(Debug)]
pub enum InputType {
    User,
    Telemetry,
    Image,
}

// input method type could be shared memory, pipe ,
pub trait Input {
    fn input_type(&self) -> InputType;
    fn event_type(&self) -> EventGame;
    fn header(&self) -> (u64, u32, u64, u32);

    // implement packet size

    // serialize input to a portable format (i.e json/xml or jpeg/png)

    // serialize to compressed format for miminal on disk size
}

//every services has a process trait, to stop and start the specific services
// all input services
pub struct Services {
    // hashmap to keep all  service objects (contain thread handle)
    //services: HashMap<ServiceType, Box<dyn Process>>,
    services_ord: Vec<(ServiceType, Box<dyn Process>)>,
}

impl Services {
    // create services objects to manage all services
    pub fn new() -> Services {
        Services {
            //services: HashMap::<ServiceType, Box<dyn Process>>::new(),
            services_ord: Vec::<(ServiceType, Box<dyn Process>)>::new(),
        }
    }
    //service needs to impliment process thread
    pub fn add_service(
        &mut self,
        service_type: ServiceType,
        mut process: Box<dyn Process>,
    ) -> Result<(), ServiceError> {
        println!("Starting serivce {:?}", service_type);
        process.start()?;
        //self.services.insert(service_type, process);
        self.services_ord.push((service_type, process));

        Ok(())
    }

    fn get_service(&mut self, service_type: ServiceType) -> Option<Box<dyn Process>> {
        let res = self
            .services_ord
            .iter()
            .position(|&(service_t, _)| service_t == service_type);

        if res.is_none() {
            None
        } else {
            Some(self.services_ord.remove(res.unwrap()).1)
        }
    }
    //stops a running service else nothing
    pub fn stop_service(&mut self, service_type: ServiceType) -> Result<(), ServiceError> {
        // if !self.services.contains_key(&service_type) {
        //     return Err(ServiceError::NotActive);
        // }

        let service = self.get_service(service_type);
        if service.is_none() {
            return Err(ServiceError::NotActive);
        }

        // let mut p = self.services.remove(&service_type).unwrap();
        let mut p = service.unwrap();
        p.stop()?;

        p.join();
        println!("service stopped {:?}", service_type);
        Ok(())
    }

    //stop all services
    pub fn stop_all_services(&mut self) -> Result<(), ServiceError> {
        for (k, v) in self.services_ord.iter_mut().rev() {
            v.stop()?;

            v.join();
            println!("service stopped {:?}", k);
        }

        Ok(())
    }

    //block until telemetry service is done
    pub fn block_until_telemetry_finished(&mut self) -> Result<(), ServiceError> {
        // if !self.services.contains_key(&ServiceType::TelemetryInput) {
        //     return Err(ServiceError::NotActive);
        // }

        let service = self.get_service(ServiceType::TelemetryInput);
        if service.is_none() {
            return Err(ServiceError::NotActive);
        }
        let mut service_handle = service.unwrap(); //self.services.remove(&ServiceType::TelemetryInput).unwrap();

        service_handle.join();

        Ok(())
    }
}
