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
            .filter_map(|o| serde_json::from_str(o).ok())
            .collect()
    }

    fn read(&self, item: Item) -> ItemMarket {
        super::read(Self::filename(item))
    }

    fn write(&self, item: Item, market: &ItemMarket) -> anyhow::Result<()> {
        super::write(Self::filename(item), market)
    }

    pub fn get(&self, item: Item) -> ItemMarket {
        self.read(item)
    }

    pub fn buy(&self, item: Item, order: Order) -> anyhow::Result<()> {
        if !order.is_valid() {
            return Err(anyhow::anyhow!("Order is invalid"));
        }
        let mut market = self.read(item);
        market.buy.push(order);
        market.sort();
        self.write(item, &market)
    }

    pub fn sell(&self, item: Item, order: Order) -> anyhow::Result<()> {
        if !order.is_valid() {
            return Err(anyhow::anyhow!("Order is invalid"));
        }
        let mut market = self.read(item);
        market.buy.push(order);
        market.sort();
        self.write(item, &market)
    }

    pub fn trade(&self) -> anyhow::Result<Vec<Trade>> {
        let items = self.list();
        let mut trades = Vec::new();
        for item in items {
            let mut market = self.read(item);
            trades.append(&mut market.resolve());
            self.write(item, &market)?;
        }
        Ok(trades)
    }
}
