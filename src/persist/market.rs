use space_game_typings::fixed::item::Item;
use space_game_typings::market::{ItemMarket, Order, Trade};

pub struct Market {}

impl Market {
    fn filename(item: Item) -> String {
        format!("persist/market/{}.yaml", item.to_string())
    }

    fn list(&self) -> Vec<Item> {
        let list = super::list("persist/market");
        list.iter()
            .filter_map(|o| o.file_stem())
            .filter_map(std::ffi::OsStr::to_str)
            .filter_map(|o| o.parse::<Item>().ok())
            .collect()
    }

    fn read(&self, item: Item) -> ItemMarket {
        super::read(Self::filename(item))
    }

    fn write(&mut self, item: Item, market: &ItemMarket) -> anyhow::Result<()> {
        super::write(Self::filename(item), market)
    }

    pub fn get(&self, item: Item) -> ItemMarket {
        self.read(item)
    }

    pub fn buy(&mut self, item: Item, order: Order) -> anyhow::Result<()> {
        if !order.is_valid() {
            return Err(anyhow::anyhow!("Order is invalid"));
        }
        let mut market = self.read(item);
        market.buy.push(order);
        market.sort();
        self.write(item, &market)
    }

    pub fn sell(&mut self, item: Item, order: Order) -> anyhow::Result<()> {
        if !order.is_valid() {
            return Err(anyhow::anyhow!("Order is invalid"));
        }
        let mut market = self.read(item);
        market.sell.push(order);
        market.sort();
        self.write(item, &market)
    }

    pub fn trade(&mut self) -> anyhow::Result<Vec<(Item, Trade)>> {
        let items = self.list();
        let mut trades = Vec::new();
        for item in items {
            let mut market = self.read(item);
            for t in market.resolve() {
                trades.push((item, t));
            }
            self.write(item, &market)?;
        }
        Ok(trades)
    }
}
