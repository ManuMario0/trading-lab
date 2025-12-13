use super::*;

fn create_mock_prices() -> Prices {
    let mut prices = Prices::default();

    // AAPL: Last 150, Bid 149, Ask 151
    let aapl = Instrument::Stock(SymbolId::new("AAPL", "NASDAQ"));
    prices.insert(aapl.clone(), Price::new(aapl, 150.0, 149.0, 151.0, 0));

    // ES Future: Last 4000, Bid 3995, Ask 4005
    let es = Instrument::Future(SymbolId::new("ES", "CME"));
    prices.insert(es.clone(), Price::new(es, 4000.0, 3995.0, 4005.0, 0));

    // EUR/USD: Last 1.10, Bid 1.09, Ask 1.11
    let eur_usd = Instrument::Forex(CurrencyPair::new("EUR", "USD"));
    prices.insert(eur_usd.clone(), Price::new(eur_usd, 1.10, 1.09, 1.11, 0));

    prices
}

#[test]
fn test_calculate_equity() {
    let prices = create_mock_prices();
    let mut portfolio = Portfolio::new();

    let aapl = Instrument::Stock(SymbolId::new("AAPL", "NASDAQ"));
    let es = Instrument::Future(SymbolId::new("ES", "CME"));

    portfolio.positions_mut().set_quantity(aapl, 10.0); // 10 * 150 = 1500
    portfolio.positions_mut().set_quantity(es, 1.0); // 1 * 4000 = 4000

    portfolio.cash_mut().deposit("USD", 1000.0);
    portfolio.cash_mut().deposit("EUR", 100.0); // 100 * 1.10 = 110

    // Expected: 1500 (AAPL) + 4000 (ES) + 1000 (USD) + 110 (EUR) = 6610
    let equity = portfolio.calculate_equity("USD", &prices);
    assert!(
        (equity - 6610.0).abs() < 1e-6,
        "Equity mismatch: {}",
        equity
    );
}

#[test]
fn test_calculate_gross_exposure() {
    let prices = create_mock_prices();
    let mut portfolio = Portfolio::new();

    let aapl = Instrument::Stock(SymbolId::new("AAPL", "NASDAQ"));
    let es = Instrument::Future(SymbolId::new("ES", "CME"));

    portfolio.positions_mut().set_quantity(aapl, -10.0); // Short 10 AAPL. | -10 | * 150 = 1500
    portfolio.positions_mut().set_quantity(es, 2.0); // Long 2 ES. 2 * 4000 = 8000

    // Expected Exposure: 1500 + 8000 = 9500
    let exposure = portfolio.calculate_gross_exposure(&prices);
    assert!(
        (exposure - 9500.0).abs() < 1e-6,
        "Exposure mismatch: {}",
        exposure
    );
}

#[test]
fn test_calculate_liquidation_value() {
    let prices = create_mock_prices();
    let mut portfolio = Portfolio::new();

    let aapl = Instrument::Stock(SymbolId::new("AAPL", "NASDAQ"));
    let es = Instrument::Future(SymbolId::new("ES", "CME"));

    // Long 10 AAPL. Sell @ Bid (149). Value = 1490.
    portfolio.positions_mut().set_quantity(aapl, 10.0);

    // Short 1 ES. Buy @ Ask (4005). Value = -1 * 4005 = -4005.
    portfolio.positions_mut().set_quantity(es, -1.0);

    portfolio.cash_mut().deposit("USD", 10000.0);

    // Net Liquidation: 1490 - 4005 + 10000 = 7485
    let liq_value = portfolio.calculate_liquidation_value("USD", &prices, 1.0);
    assert!(
        (liq_value - 7485.0).abs() < 1e-6,
        "Liquidation Value mismatch: {}",
        liq_value
    );
}

#[test]
fn test_instrument_serialization() {
    let inst = Instrument::Stock(SymbolId::new("AAPL", "NASDAQ"));
    let json = serde_json::to_string(&inst).unwrap();
    println!("Serialized Stock: {}", json);
    assert!(json.contains("\"type\":\"Stock\""));
    assert!(json.contains("\"symbol\":\"AAPL\""));
}
