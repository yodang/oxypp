pub mod oxypp{
    use tungstenite::{Message, WebSocket, accept};
    use tungstenite::stream::MaybeTlsStream;
    use std::net::{TcpStream, TcpListener, ToSocketAddrs};
    use std::io::{Read, Write};
    use serde::{Serialize, Deserialize};
    use regex::Regex;

    #[derive(PartialEq, Debug)]
    pub enum Error{
        GeneralError
    }

    type Result<T>=std::result::Result<T, Error>;

    #[allow(non_snake_case)]
    #[derive(Serialize,Deserialize,Debug)]
    struct BootNotificationReq{
        chargeBoxSerialNumber: Option<String>,
        chargePointModel: String,
        chargePointSerialNumber: Option<String>,
        chargePointVendor:String,
        firmwareVersion:Option<String>,
        iccid:Option<String>,
        imsi:Option<String>,
        meterSerialNumber:Option<String>,
        meterType:Option<String>
    }

    #[derive(PartialEq, Debug)]
    pub enum MessageTypeId{
        Call=2,
        CallResult=3,
        CallError=4
    }

    #[derive(PartialEq, Debug)]
    pub enum OCPPAction{
        BootNotification
    }

    #[derive(PartialEq, Debug)]
    pub enum OCPPError{
        NotImplemented,
        NotSupported,
        InternalError,
        ProtocolError,
        SecurityError,
        FormationViolation,
        PropertyConstraintViolation,
        OccurenceConstraintViolation,
        TypeConstraintViolation,
        GenericError
    }

    impl OCPPError{
        fn from_str(s:&str)->std::result::Result<Self,()>
        {
            match s{
                "NotImplemented"=>Ok(OCPPError::NotImplemented),
                "NotSupported"=>Ok(OCPPError::NotSupported),
                "InternalError"=>Ok(OCPPError::InternalError),
                "ProtocolError"=>Ok(OCPPError::ProtocolError),
                "SecurityError"=>Ok(OCPPError::SecurityError),
                "FormationViolation"=>Ok(OCPPError::FormationViolation),
                "PropertyConstraintViolation"=>Ok(OCPPError::PropertyConstraintViolation),
                "OccurenceConstraintViolation"=>Ok(OCPPError::OccurenceConstraintViolation),
                "TypeConstraintViolation"=>Ok(OCPPError::TypeConstraintViolation),
                "GenericError"=>Ok(OCPPError::GenericError),
                _=>Err(())
            }
        }
    }

    impl OCPPAction{
        fn from_str(s:&str)->std::result::Result<Self,()>
        {
            match s{
                "BootNotification"=>Ok(OCPPAction::BootNotification),
                _=>Err(())
            }
        }
    }

    #[derive(PartialEq, Debug)]
    pub enum OCPPMessage{
        Call{id:u32,message_type:OCPPAction,payload:String},
        CallResult{id:u32,payload:String},
        CallError{id:u32,error:OCPPError,description:String,details:String}
    }

    impl From<OCPPMessage> for Message{
        fn from(m: OCPPMessage)->Self
        {
            match m {
                OCPPMessage::Call{id,message_type,payload}=>{
                    Message::Text(format!("[{},{},{:?},{}]", /*MessageTypeId::Call*/2, id, message_type, payload))
                }
                OCPPMessage::CallResult{id, payload}=>{
                    Message::Text(format!("[{},{},{}]", 3, id, payload))
                }
                OCPPMessage::CallError{id, error, description, details}=>{
                    Message::Text(format!("[{},{},{:?},{},{}]", 4, id, error, description, details))
                }
            }
        }
    }

    pub fn parse_message(m: &str)->Result<OCPPMessage>
    {
        let regexp_call=Regex::new(r#"^\[\s*2\s*,\s*"?([\d]+)"?\s*,\s*"([[:alpha:]]+)"\s*,\s*((?s)\{.*\})\s*\]$"#).unwrap();
        let regexp_result=Regex::new(r#"^\[\s*3\s*,\s*"?([\d]+)"?\s*,\s*((?s)\{.*\})\s*\]$"#).unwrap();
        let regexp_error=Regex::new(r#"^\[\s*4\s*,\s*"?([\d]+)"?\s*,\s*"([[:alpha:]]+)"\s*,\s*"([[:alpha:]\s]*)",\s*((?s)\{.*\})\s*\]$"#).unwrap();
        if let Some(capture)=regexp_call.captures(m)
        {
            if let Ok(action)=OCPPAction::from_str(capture.get(2).unwrap().as_str())
            {
                Ok(OCPPMessage::Call{
                    id:capture.get(1).unwrap().as_str().parse().unwrap(),
                    message_type:action,
                    payload:capture.get(3).unwrap().as_str().to_owned()
                })
            }
            else {Err(Error::GeneralError)}
        }
        else if let Some(capture)=regexp_result.captures(m)
        {
            Ok(OCPPMessage::CallResult{
                id:capture.get(1).unwrap().as_str().parse().unwrap(),
                payload:capture.get(2).unwrap().as_str().to_owned()})
        }
        else if let Some(capture)=regexp_error.captures(m)
        {
            if let Ok(error_code)=OCPPError::from_str(capture.get(2).unwrap().as_str()) {
                Ok(OCPPMessage::CallError{
                    id:capture.get(1).unwrap().as_str().parse().unwrap(),
                    error:error_code,
                    description:capture.get(3).unwrap().as_str().to_owned(),
                    details:capture.get(4).unwrap().as_str().to_owned()
                })
            }
            else{
                Err(Error::GeneralError)
            }
        }
        else {Err(Error::GeneralError)}
    }

    pub struct CPContext{
        sock: WebSocket<MaybeTlsStream<TcpStream>>
    }

    impl CPContext{
        pub fn open(address: &str)->Result<Self>
        {
            match tungstenite::connect(address){
                Ok((ws, _)) =>
                {
                    Ok(CPContext{sock:ws})
                }
                Err(e)=>
                {
                    println!("Error: {}", e);
                    Err(Error::GeneralError)
                }
            }
        }
        pub fn authorize()
        {
        }
        pub fn boot_notification()
        {
        }
        pub fn data_transfer()
        {
        }
        pub fn start_transaction()
        {
        }
        pub fn read_next_message(&mut self)->Result<OCPPMessage>
        {
            if let Ok(m)=self.sock.read_message()
            {
                parse_message(m.to_text().unwrap_or(""))
            }
            else
            {
                Err(Error::GeneralError)
            }
        }
    }

    pub struct CSContext{
        sock: TcpListener
    }

    impl CSContext{
        pub fn bind<A: ToSocketAddrs>(addr: A)->Result<Self>{
            if let Ok(listener) = TcpListener::bind(addr)
            {
                return Ok(CSContext{sock:listener})
            }
            else
            {
                return Err(Error::GeneralError);
            }
        }
        pub fn accept(&self) -> Result<OCPPStream>
        {
            if let Ok((stream,_))=self.sock.accept()
            {
                if let Ok(ws)=tungstenite::accept(stream)
                {
                    return Ok(OCPPStream{websocket:ws});
                }
                else
                {
                    return Err(Error::GeneralError);
                }
            }
            else
            {
                return Err(Error::GeneralError);
            }
        }
    }

    pub struct OCPPStream{
        websocket:WebSocket<TcpStream>
    }

    impl OCPPStream {
        pub fn next_call(&mut self) -> Result<OCPPMessage>
        {
            if let Ok(m)=self.websocket.read_message()
            {
                parse_message(m.to_text().unwrap_or(""))
            }
            else
            {
                Err(Error::GeneralError)
            }
        }

        pub fn send_response(&mut self, m: OCPPMessage)->Result<()>
        {
            match self.websocket.write_message(Message::from(m))
            {
                Ok(())=>
                {
                    Ok(())
                }
                //Err(tungstenite::Error::SendQueueFull(_))=>
                //{
                //}
                _=>
                {
                    Err(Error::GeneralError)
                }
            }
        }
    }

}


#[cfg(test)]
mod oxypp_tests {
    use crate::oxypp::*;
    #[test]
    fn parsing_request() {
        let result = parse_message(r#"[2, "12345", "BootNotification", {}]"#);
        assert_eq!(result, Ok(OCPPMessage::Call{
            id:12345,
            message_type:OCPPAction::BootNotification,
            payload:"{}".to_owned()
        }));
    }
    #[test]
    fn parsing_response() {
        let result = parse_message(r#"[3,
        "19223201",
        {"status":"Accepted", "currentTime":"2013-02-01T20:53:32.486Z", "heartbeatInterval":300}
       ]"#);
        assert_eq!(result, Ok(OCPPMessage::CallResult{
            id:19223201,
            payload:r#"{"status":"Accepted", "currentTime":"2013-02-01T20:53:32.486Z", "heartbeatInterval":300}"#.to_owned()
        }));
    }
    #[test]
    fn parsing_error() {
        let result = parse_message(r#"[4, "12345", "NotImplemented", "MyError", {}]"#);
        assert_eq!(result, Ok(OCPPMessage::CallError{
            id:12345,
            error:OCPPError::NotImplemented,
            description:"MyError".to_owned(),
            details:"{}".to_owned()

        }));
    }
    #[test]
    fn create_cp(){
        let cs=CSContext::bind("127.0.0.1:8008").unwrap();
        let th=std::thread::spawn(move ||{
            let _res=cs.accept();
        });
        //std::thread::sleep_ms(1000);
        let _cp=CPContext::open("ws://127.0.0.1:8008").unwrap();
        th.join().unwrap_or(());
    }
    #[test]
    fn create_cs(){
        let _cs=CSContext::bind("127.0.0.1:8008").unwrap();
    }
}
