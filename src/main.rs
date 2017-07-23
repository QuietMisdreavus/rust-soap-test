#[macro_use] extern crate hyper;
extern crate sulfate_xml;

//mod elem;

use sulfate_xml::{ToXml, FromXml, Element, ElemContent};

use std::fmt;
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
///The XML namespace URL for IService-specific elements.
const SVC_NS: &'static str = "http://tempuri.org/";

struct SoapEnvelope<T>(T);

impl<T: ToXml> ToXml for SoapEnvelope<T> {
    fn to_xml(&self) -> Element {
        let mut envelope = Element::new_ns_prefix("Envelope", SOAP_NS, "s");
        let mut body = Element::new_ns_prefix("Body", SOAP_NS, "s");
        body.push_child(self.0.to_xml());
        envelope.push_child(body);
        envelope
    }
}

#[derive(Debug)]
enum SoapError<T> {
    NotSoapEnvelope,
    EmptyBody,
    Other(T),
}

impl<T> From<T> for SoapError<T> {
    fn from(src: T) -> SoapError<T> {
        SoapError::Other(src)
    }
}

impl<T: fmt::Debug + fmt::Display> Error for SoapError<T> {
    fn description(&self) -> &str {
        match *self {
            SoapError::NotSoapEnvelope => "Invalid XML parsed as SOAP envelope",
            SoapError::EmptyBody => "SOAP envelope parsed with empty body",
            SoapError::Other(_) => "Error occured while parsing contents of SOAP envelope",
        }
    }
}

impl<T: fmt::Display + fmt::Debug> fmt::Display for SoapError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            SoapError::Other(ref inner) => {
                write!(f, "Error while parsing contents of SOAP envelope: {}", inner)
            }
            ref other => {
                write!(f, "{}", <Self as Error>::description(other))
            }
        }
    }
}

impl<T: FromXml> FromXml for SoapEnvelope<T> {
    type Error = SoapError<T::Error>;

    fn from_xml(src: &Element) -> Result<Self, SoapError<T::Error>> {
        //NOTE: like the names in the ToXml impl, these names and namespaces are SOAP standard
        if src.name.local_name != "Envelope" || src.name.namespace != Some(SOAP_NS.into()) {
            return Err(SoapError::NotSoapEnvelope);
        }

        let body = src.first_child_where(|bod| bod.name.local_name == "Body" &&
                                               bod.name.namespace == Some(SOAP_NS.into()));
        if body.is_none() {
            return Err(SoapError::NotSoapEnvelope);
        }

        if let Some(inner) = body.and_then(|bod| bod.first_child_where(|_| true)) {
            Ok(SoapEnvelope(T::from_xml(inner)?))
        } else {
            Err(SoapError::EmptyBody)
        }
    }
}

struct GetDataRequest {
    value: i32,
}

impl ToXml for GetDataRequest {
    fn to_xml(&self) -> Element {
        //NOTE: the element names and namespaces are yanked from the WSDL
        let mut ret = sulfate_xml::Element::new_default_ns("GetData", SVC_NS);
        let mut value = sulfate_xml::Element::new("value");
        value.push_text(self.value.to_string());
        ret.push_child(value);
        ret
    }
}

struct GetDataResponse {
    get_data_result: String,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct IServiceError;

impl fmt::Display for IServiceError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Error occurred while parsing IService response")
    }
}

impl FromXml for GetDataResponse {
    type Error = IServiceError;

    fn from_xml(src: &Element) -> Result<Self, IServiceError> {
        if src.name.local_name != "GetDataResponse" ||
           src.name.namespace != Some(SVC_NS.into())
        {
            return Err(IServiceError);
        }

        let result = src.first_child_where(|res| res.name.local_name == "GetDataResult" &&
                                                 res.name.namespace == Some(SVC_NS.into()));

        match result.and_then(|r| r.content.first()) {
            Some(&ElemContent::Text(ref text)) => Ok(GetDataResponse {
                get_data_result: text.clone().into_owned(),
            }),
            _ => Err(IServiceError),
        }
    }
}

fn get_data(value: i32) -> Result<String, Box<Error>> {
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
    println!();
    println!("{:#}", out);

    let ret = SoapEnvelope::<GetDataResponse>::from_xml(&out)?;

    Ok(ret.0.get_data_result)
}

fn main() {
    let resp = get_data(33).unwrap();

    println!();
    println!("{}", resp);
}
