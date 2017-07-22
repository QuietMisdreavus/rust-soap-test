#[macro_use] extern crate hyper;
extern crate sulfate_xml;

//mod elem;

use sulfate_xml::{ToXml, Element};

use std::io::Write;
use std::error::Error;

use hyper::client::{Client, Body};
use hyper::header::ContentType;

header! { (SoapAction, "SOAPAction") => [String] }

//NOTE: this is the namespace url for SOAP standard stuff
const SOAP_NS: &'static str = "http://schemas.xmlsoap.org/soap/envelope/";
const CONTENT_XML: &'static str = "text/xml; charset=utf-8";

//NOTE: these URLs are yanked from the WSDL

//wsdl:definitions/wsdl:service/wsdl:port/soap:address@location
///The URL to POST to interact with the service.
const SERVICE_URL: &'static str = "http://localhost:53016/Service1.svc";
//wsdl:definitions/wsdl:binding/wsdl:operation[name="GetData"]/soap:operation@soapAction
///The URL to use as a `SoapAction` when performing a `GetData`.
const ACTION: &'static str = "http://tempuri.org/IService1/GetData";

struct SoapEnvelope<T: ToXml>(T);

impl<T: ToXml> ToXml for SoapEnvelope<T> {
    fn to_xml(&self) -> Element {
        let mut envelope = Element::new_ns_prefix("Envelope", SOAP_NS, "s");
        let mut body = Element::new_ns_prefix("Body", SOAP_NS, "s");
        body.push_child(self.0.to_xml());
        envelope.push_child(body);
        envelope
    }
}

struct GetDataRequest {
    value: i32,
}

impl ToXml for GetDataRequest {
    fn to_xml(&self) -> Element {
        //NOTE: the element names and namespaces are yanked from the WSDL
        let mut ret = sulfate_xml::Element::new_default_ns("GetData", "http://tempuri.org/");
        let mut value = sulfate_xml::Element::new("value");
        value.push_text(self.value.to_string());
        ret.push_child(value);
        ret
    }
}

fn get_data(value: i32) -> Result<Element<'static>, Box<Error>> {
    let mut buf: Vec<u8> = Vec::new();
    let msg = SoapEnvelope(GetDataRequest{ value });
    let msg = msg.to_xml();
    println!("{:#}", msg);
    write!(buf, "{}", msg)?;

    let client = Client::new();
    let body = Body::BufBody(&buf, buf.len());
    let resp = client.post(SERVICE_URL)
                         .body(body)
                         .header(ContentType(CONTENT_XML.parse().unwrap()))
                         .header(SoapAction(ACTION.to_string()))
                         .send()?;

    let out = Element::from_stream(resp)?;

    Ok(out)
}

fn main() {
    let resp = get_data(33).unwrap();

    println!();
    println!("{:#}", resp);
}
