
# TODO
    - implement keyboard/controller input capture
    - implement logging
    - standardized input data into portable output format i.e json for telemetry and keyboard/controller and jpeg for screencapture


# Implemented with test cases
    - shared_memory telemetry serivce for ets2 emulation - interrupt shutdown and auto shutdown
    - synchronization service 
        - groupify: first index and last index in vector are FrameStartEvent and FrameEndEvent respectively everything else is in between. 
        - testing with the following services telementry and screencapture 


# Services
    - responsible for starting  and stoping  services
    - all services are execute in a separete thread and transfer data using message passing channels

## Synchronication Service
    - responsible for synchronizing all the input services into Vectors where the first item is the start frame telemetry packet and last item is the end frame telemetry packet.
      all other input services packets falls between them.
     
## Telemetry Services
    - responsible for retrieving game data
    - added functionality to retrieve game data from ETS2 using shared memory

## ScreenCapture Service
    - responsible for capturing the main monitor 
    - added functionality to capture screenshot using Windows desktopduplication API. 

## Keyboard/Controller Service
    - Not implemented yet


    
