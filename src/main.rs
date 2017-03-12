#[macro_use] extern crate hyper;
extern crate xml;

use std::io::{Read, Write};
use std::error::Error;

use hyper::client::{Client, Body};
use hyper::header::ContentType;
use xml::writer::{self, EventWriter, XmlEvent};

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

fn soap_envelope<W, F>(sink: W, mut body: F) -> writer::Result<W>
    where W: Write,
          F: FnMut(&mut EventWriter<W>) -> writer::Result<()>
{
    let mut sink = EventWriter::new(sink);

    //NOTE: the element names and namespaces here are SOAP standard
    sink.write(XmlEvent::start_element("s:Envelope").ns("s", SOAP_NS))?;
    sink.write(XmlEvent::start_element("s:Body"))?;
    body(&mut sink)?;
    sink.write(XmlEvent::end_element())?;
    sink.write(XmlEvent::end_element())?;

    Ok(sink.into_inner())
}

fn get_data_body<W: Write>(sink: &mut EventWriter<W>, value: i32) -> writer::Result<()> {
    //NOTE: the element names and namespaces are yanked from the WSDL
    sink.write(XmlEvent::start_element("GetData").default_ns("http://tempuri.org/"))?;
    sink.write(XmlEvent::start_element("value"))?;
    sink.write(value.to_string().as_str())?;
    sink.write(XmlEvent::end_element())?;
    sink.write(XmlEvent::end_element())?;

    Ok(())
}

fn get_data(value: i32) -> Result<String, Box<Error>> {
    let buf: Vec<u8> = Vec::new();
    let buf = soap_envelope(buf, |sink| get_data_body(sink, value))?;

    let client = Client::new();
    let body = Body::BufBody(&buf, buf.len());
    let mut resp = client.post(SERVICE_URL)
                         .body(body)
                         .header(ContentType(CONTENT_XML.parse().unwrap()))
                         .header(SoapAction(ACTION.to_string()))
                         .send()?;

    //TODO: actually parse the XML output >_>
    let mut out = String::new();
    resp.read_to_string(&mut out)?;

    Ok(out)
}

fn main() {
    let resp = get_data(33).unwrap();

    println!("{}", resp);
}
