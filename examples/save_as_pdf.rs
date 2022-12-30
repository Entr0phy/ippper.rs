use async_trait::async_trait;
use ippper::server::IppServer;
use ippper::service::{
    PrinterInfoBuilder, SimpleIppDocument, SimpleIppService, SimpleIppServiceHandler,
};
use ipp::value::IppValue;
use ipp::model::DelimiterTag;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::fs::File;
use tokio::io;
use tokio_util::compat::*;
use uuid::Uuid;

struct MyHandler {}
impl MyHandler {
    fn new() -> Self {
        Self {}
    }
}
#[async_trait]
impl SimpleIppServiceHandler for MyHandler {
    async fn handle_document(&self, document: SimpleIppDocument) -> anyhow::Result<()> {
        let sender:Option<String> = document.job_attr
            .groups_of(DelimiterTag::OperationAttributes)
            .next()
            .and_then(|g| g.attributes().get("requesting-user-name"))
            .map(|attr| attr.value())
            .and_then(|attr| match attr {
                IppValue::NameWithoutLanguage(x) => Some(x.clone()),
                _ => None,
            });


        println!("Received file from {:?}", sender);
        let mut file = File::create("D:\\1.pdf").await?;
        io::copy(&mut document.payload.compat(), &mut file).await?;
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 42069);
    let mut ipp_handler = SimpleIppService::new(MyHandler::new());
    ipp_handler.set_info(
        PrinterInfoBuilder::default()
            .uuid(Some(
                // Change it if you are building a ipp service
                // Make it unique for each instance
                Uuid::parse_str("786a551c-65a3-43ce-89ba-33c51bae9bc2").unwrap(),
            ))
            .build()
            .unwrap(),
    );
    if let Err(e) = IppServer::serve(addr, Arc::new(ipp_handler)).await {
        eprintln!("server error: {}", e);
    }
}
