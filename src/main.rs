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

    let cpu_controller = 3;

    let mem_controller_ids = [0u32, 1];

    let mem_controller_lengths =
        futures::future::join_all(mem_controller_ids.iter().map(|id| async {
            let controller = client.get_controller(*id).await?;
            anyhow::Ok(controller.leds.len())
        }))
        .await
        .into_iter()
        .collect::<anyhow::Result<Vec<_>>>()?;

    let mem = sys.memory()?;
    let mem2 = mem.platform_memory.meminfo;
    println!("{mem2:?}");

    let mut interval = tokio::time::interval(Duration::from_millis(20));
    let mut cpu = sys.cpu_load_aggregate()?;
    loop {
        let meas = cpu.done()?;
        cpu = sys.cpu_load_aggregate()?;
        let mem = sys.memory()?;

        let mem_pct = mem.platform_memory.meminfo["Active(anon)"].0 as f64 / mem.total.0 as f64;
        for (id, len) in mem_controller_ids.iter().zip(mem_controller_lengths.iter()) {
            let draw_len = mem_pct * (*len as f64);
            let floor = draw_len.floor() as usize;
            let mut brightness = vec![0.0f64; *len];
            brightness[0..floor].fill(1.0);
            brightness[floor] = draw_len - draw_len.floor();
            let colors = brightness
                .into_iter()
                .map(|c| RGB8::new(0, (c * 256.0) as u8, 0))
                .collect();
            client.update_leds(*id, colors).await?;
        }

        // let colors = meas
        //     .iter()
        //     .map(|m| 1.0 - m.idle)
        //     .map(|c| RGB8::new((c * 256.0) as u8, 0, 0))
        //     .collect();
        // client.update_leds(cpu_controller, colors).await?;
        client
            .update_led(
                cpu_controller,
                2,
                RGB8::new(((1.0 - meas.idle) * 256.0) as u8, 0, 0),
            )
            .await?;

        interval.tick().await;
    }
}
