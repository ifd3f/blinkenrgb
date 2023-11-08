use openrgb::OpenRGB;
use rgb::RGB8;
use systemstat::{Duration, Platform};
use tokio::{net::TcpStream, time::Interval};

#[tokio::main]
pub async fn main() {
    let client = OpenRGB::connect()
        .await
        .expect("failed to connect to OpenRGB");

    doit(client).await.unwrap();
}

async fn doit(client: OpenRGB<TcpStream>) -> anyhow::Result<()> {
    client.set_name("BlinkenRGB").await?;

    let count = client.get_controller_count().await.unwrap();
    let sys = systemstat::System::new();

    let controller_id = 3;

    let mut interval = tokio::time::interval(Duration::from_millis(50));
    let mut cpu = sys.cpu_load()?;
    loop {
        let meas = cpu.done()?;
        cpu = sys.cpu_load()?;

        let colors = meas
            .iter()
            .map(|m| 1.0 - m.idle)
            .map(|c| RGB8::new((c * 256.0) as u8, 0, 0))
            .collect();
        client.update_leds(controller_id, colors).await?;

        interval.tick().await;
    }
}
