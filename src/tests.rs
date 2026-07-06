#[cfg(test)]
mod tests {
    use crate::*;

    fn assert_series_eq(left: &[f64], right: &[f64]) {
        assert_eq!(
            left.len(),
            right.len(),
            "series lengths differ: {} vs {}",
            left.len(),
            right.len()
        );
        for (i, (l, r)) in left.iter().zip(right.iter()).enumerate() {
            if l.is_nan() && r.is_nan() {
                continue;
            }
            assert!(
                (l - r).abs() < 1e-10,
                "series differ at index {}: left={}, right={}",
                i,
                l,
                r
            );
        }
    }

    macro_rules! assert_vec_eq {
        ($left:expr, $right:expr) => {
            assert_series_eq(&$left, &$right);
        };
    }

    fn assert_outputs_eq(left: &[IndicatorOutput], right: &[IndicatorOutput], names: &[&str]) {
        for name in names {
            let l = left
                .iter()
                .find(|o| o.name == *name)
                .expect(&format!("left missing {}", name));
            let r = right
                .iter()
                .find(|o| o.name == *name)
                .expect(&format!("right missing {}", name));
            assert_series_eq(&l.values, &r.values);
        }
    }

    fn bars(closes: &[f64]) -> Vec<Bar> {
        closes
            .iter()
            .enumerate()
            .map(|(i, close)| Bar {
                time: i as u32,
                open: *close,
                high: *close,
                low: *close,
                close: *close,
                volume: 1.0,
            })
            .collect()
    }

    fn store_from_bars(bars: Vec<Bar>) -> CandleStore {
        CandleStore::from_bars(bars)
    }

    #[test]
    fn candle_store_from_columns_matches_from_bars() {
        let bars = bars(&[10.0, 11.0, 12.0]);
        let from_bars = CandleStore::from_bars(bars.clone());
        let from_columns = CandleStore::from_columns(CandleColumnsInput {
            time: bars.iter().map(|bar| bar.time).collect(),
            open: bars.iter().map(|bar| bar.open).collect(),
            high: bars.iter().map(|bar| bar.high).collect(),
            low: bars.iter().map(|bar| bar.low).collect(),
            close: bars.iter().map(|bar| bar.close).collect(),
            volume: bars.iter().map(|bar| bar.volume).collect(),
        })
        .unwrap();

        assert_eq!(from_columns.time, from_bars.time);
        assert_eq!(from_columns.open, from_bars.open);
        assert_eq!(from_columns.high, from_bars.high);
        assert_eq!(from_columns.low, from_bars.low);
        assert_eq!(from_columns.close, from_bars.close);
        assert_eq!(from_columns.volume, from_bars.volume);
    }

    #[test]
    fn candle_store_from_columns_rejects_mismatched_lengths() {
        let error = CandleStore::from_columns(CandleColumnsInput {
            time: vec![0, 1],
            open: vec![1.0],
            high: vec![1.0, 2.0],
            low: vec![1.0, 2.0],
            close: vec![1.0, 2.0],
            volume: vec![1.0, 2.0],
        })
        .err()
        .unwrap();

        assert_eq!(
            Some(error),
            Some("candle column lengths must match for time/open/high/low/close/volume")
        );
    }

    fn ohlc(values: &[(f64, f64, f64)]) -> Vec<Bar> {
        values
            .iter()
            .enumerate()
            .map(|(i, (high, low, close))| Bar {
                time: i as u32,
                open: *close,
                high: *high,
                low: *low,
                close: *close,
                volume: 1.0,
            })
            .collect()
    }

    fn indicator_stub(kind: &str) -> Indicator {
        Indicator {
            id: 1,
            kind: kind.to_string(),
            period: 14,
            stoch_period: 14,
            smooth: 28,
            signal: 9,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            anchor: 0,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        }
    }

    #[test]
    fn sma_waits_for_period_then_rolls() {
        assert_vec_eq!(
            sma(&bars(&[1.0, 2.0, 3.0, 4.0]), 3),
            vec![f64::NAN, f64::NAN, 2.0, 3.0]
        );
    }

    #[test]
    fn ema_updates_from_first_close() {
        assert_eq!(ema(&bars(&[10.0, 12.0, 14.0]), 3), vec![10.0, 11.0, 12.5]);
    }

    #[test]
    #[test]
    #[test]
    fn store_sma_matches_row_sma() {
        let bars = bars(&[1.0, 2.0, 3.0, 4.0, 5.0]);
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(
            sma(&bars, 3),
            sma_close_store(&store, 3, &mut HashMap::new())
        );
        assert_eq!(latest_sma(&bars, 3), latest_sma_store(&store, 3));
    }

    #[test]
    fn store_ema_matches_row_ema() {
        let bars = bars(&[10.0, 12.0, 14.0, 13.0]);
        let store = store_from_bars(bars.clone());
        let previous_output = IndicatorOutput {
            name: "value".to_string(),
            values: ema(&bars[..bars.len() - 1], 3),
        };

        assert_eq!(
            ema(&bars, 3),
            *ema_close_store(&store, 3, &mut HashMap::new())
        );
        assert_eq!(
            latest_ema(&bars, 3, Some(&previous_output.values[..])),
            latest_ema_store(&store, 3, Some(&previous_output.values[..]))
        );
    }

    #[test]
    fn rsi_waits_for_period_then_uses_wilder_smoothing() {
        let values = rsi(&bars(&[1.0, 2.0, 1.0, 3.0, 2.0]), 3);
        assert!(values[0].is_nan());
        assert!(values[1].is_nan());
        assert!(values[2].is_nan());
        assert_eq!(values[3], 75.0);
        assert!((values[4] - 54.54545454545455).abs() < 0.000001);
    }

    #[test]
    #[test]
    fn hidden_state_outputs_are_not_visible() {
        assert!(is_visible_output("value"));
        assert!(is_visible_output("histogram"));
        assert!(!is_visible_output("avg_gain"));
        assert!(!is_visible_output("tr_avg"));
        assert!(!is_visible_output("fast_ema"));
        assert!(!is_visible_output("cumulative_pv"));
    }

    #[test]
    fn all_exposed_indicators_support_incremental_updates() {
        for descriptor in indicator_descriptors() {
            assert!(
                supports_incremental(descriptor.kind),
                "{} must be handled incrementally or intentionally hidden",
                descriptor.kind
            );
        }
    }

    #[test]
    fn unknown_indicator_does_not_support_incremental_updates() {
        assert!(!supports_incremental("UNKNOWN"));
    }

    #[test]
    fn rsi_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "RSI".to_string(),
            period: 14,
            stoch_period: 14,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            anchor: 0,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["rsi:close:14"]);
    }

    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    fn wma_has_a_computed_dag_node() {
        let indicator = Indicator {
            id: 1,
            kind: "WMA".to_string(),
            period: 20,
            stoch_period: 20,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            anchor: 0,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(indicator_nodes(&indicator), vec!["wma:close:20"]);
    }

    #[test]
    fn hma_has_computed_dag_nodes() {
        let indicator = Indicator {
            id: 1,
            kind: "HMA".to_string(),
            period: 20,
            stoch_period: 20,
            smooth: 3,
            signal: 3,
            tenkan_period: 9,
            kijun_period: 26,
            senkou_b_period: 52,
            macd: None,
            multiplier: 2.0,
            psar_step: 0.02,
            psar_max_step: 0.2,
            anchor: 0,
            outputs: IndicatorArena::from_outputs(Vec::new()),
        };

        assert_eq!(
            indicator_nodes(&indicator),
            vec!["wma:close:10", "wma:close:20", "hma:close:20"]
        );
    }

    #[test]
    #[test]
    fn dema_has_computed_dag_nodes() {
        let mut indicator = indicator_stub("DEMA");
        indicator.period = 15;
        assert_eq!(
            indicator_nodes(&indicator),
            vec!["ema:close:15", "dema:ema2:15", "dema:value:15"]
        );
    }

    #[test]
    fn tema_has_computed_dag_nodes() {
        let mut indicator = indicator_stub("TEMA");
        indicator.period = 15;
        assert_eq!(
            indicator_nodes(&indicator),
            vec![
                "ema:close:15",
                "tema:ema2:15",
                "tema:ema3:15",
                "tema:value:15"
            ]
        );
    }

    #[test]
    fn trima_has_computed_dag_nodes() {
        let mut indicator = indicator_stub("TRIMA");
        indicator.period = 20;
        assert_eq!(
            indicator_nodes(&indicator),
            vec!["sma:close:20", "trima:value:20"]
        );
    }

    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    fn store_volume_indicators_match_row_versions() {
        let mut bars = ohlc(&[(3.0, 0.0, 0.0), (6.0, 0.0, 0.0), (8.0, 1.0, 5.0)]);
        bars[0].volume = 2.0;
        bars[1].volume = 4.0;
        bars[2].volume = 3.0;
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(obv(&bars), &obv_store(&store, &mut HashMap::new()));
        assert_vec_eq!(*adl_store(&store, &mut HashMap::new()), &adl_store(&store, &mut HashMap::new()));
        assert_vec_eq!(vwma(&bars, 2), &vwma_store(&store, 2, &mut HashMap::new()));
        let row_vwap = vwap(&bars, &mut HashMap::new());
        let store_vwap = vwap_store(&store, &mut HashMap::new());
        assert_outputs_eq(
            &row_vwap,
            &store_vwap,
            &["value", "cumulative_pv", "cumulative_volume"],
        );
    }

    #[test]
    fn store_window_indicators_match_row_versions() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
            (14.0, 12.0, 13.0),
        ]);
        for (bar, volume) in bars.iter_mut().zip([1.0, 2.0, 3.0, 4.0, 5.0]) {
            bar.volume = volume;
        }
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(roc(&bars, 2), &roc_store(&store, 2, &mut HashMap::new()));
        assert_vec_eq!(*cmf_store(&store, 3, &mut HashMap::new()), &cmf_store(&store, 3, &mut HashMap::new()));
        let row_bb = bollinger(&bars, 3, 2.0, &mut HashMap::new());
        let store_bb = bollinger_store(&store, 3, 2.0, &mut HashMap::new());
        assert_outputs_eq(&row_bb, &store_bb, &["upper", "middle", "lower"]);
    }

    #[test]
    fn store_oscillator_windows_match_row_versions() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
            (14.0, 12.0, 13.0),
            (15.0, 13.0, 14.0),
        ]);
        for (bar, volume) in bars.iter_mut().zip([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]) {
            bar.volume = volume;
        }
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(cci(&bars, 3), &cci_store(&store, 3, &mut HashMap::new()));
        assert_vec_eq!(
            williams_r(&bars, 3),
            williams_r_store(&store, 3, &mut HashMap::new())
        );
        assert_vec_eq!(mfi(&bars, 3), &mfi_store(&store, 3, &mut HashMap::new()));
        let row_stoch = stochastic(&bars, 3, 2, &mut HashMap::new());
        let store_stoch = stochastic_store(&store, 3, 2, &mut HashMap::new());
        assert_outputs_eq(&row_stoch, &store_stoch, &["k", "d"]);
    }

    #[test]
    fn store_stateful_ohlc_indicators_match_row_versions() {
        let bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
            (14.0, 12.0, 13.0),
            (15.0, 13.0, 14.0),
            (16.0, 14.0, 15.0),
            (17.0, 15.0, 16.0),
            (18.0, 16.0, 17.0),
        ]);
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(atr(&bars, 3), &atr_store(&store, 3, &mut HashMap::new()));
        let row_adx = adx(&bars, 3, &mut HashMap::new());
        let store_adx = adx_store(&store, 3, &mut HashMap::new());
        assert_outputs_eq(
            &row_adx,
            &store_adx,
            &[
                "value",
                "plus_di",
                "minus_di",
                "tr_avg",
                "plus_dm_avg",
                "minus_dm_avg",
                "dx",
            ],
        );
        for (row, store_output) in [
            (
                keltner(&bars, 3, 2.0, &mut HashMap::new()),
                keltner_store(&store, 3, 2.0, &mut HashMap::new()),
            ),
            (
                starc(&bars, 3, 2.0, &mut HashMap::new()),
                starc_store(&store, 3, 2.0, &mut HashMap::new()),
            ),
            (
                supertrend(&bars, 3, 2.0, &mut HashMap::new()),
                supertrend_store(&store, 3, 2.0, &mut HashMap::new()),
            ),
        ] {
            for row_output in &row {
                let store_vals = store_output
                    .iter()
                    .find(|o| o.name == row_output.name)
                    .map(|o| &o.values)
                    .unwrap();
                assert_series_eq(&row_output.values, store_vals);
            }
        }
    }

    #[test]
    fn store_misc_incremental_batch_matches_row_versions() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
            (14.0, 12.0, 13.0),
            (15.0, 13.0, 14.0),
        ]);
        for (bar, volume) in bars.iter_mut().zip([1.0, 2.0, 3.0, 4.0, 5.0, 6.0]) {
            bar.volume = volume;
        }
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(wma(&bars, 3), &wma_store(&store, 3, &mut HashMap::new()));
        assert_vec_eq!(dpo(&bars, 4), &dpo_store(&store, 4, &mut HashMap::new()));
        assert_vec_eq!(
            force_index(&bars, 2),
            force_index_store(&store, 2, &mut HashMap::new())
        );
        let row_channel = price_channel(&bars, 3, &mut HashMap::new());
        let store_channel = price_channel_store(&store, 3, &mut HashMap::new());
        assert_outputs_eq(&row_channel, &store_channel, &["upper", "middle", "lower"]);
    }

    #[test]
    fn store_math_batch_matches_row_versions() {
        let bars = bars(&[10.0, 12.0, 11.0, 13.0, 15.0, 14.0, 16.0, 18.0]);
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(
            hma(&bars, 5, &mut HashMap::new()),
            hma_store(&store, 5, &mut HashMap::new())
        );
        assert_vec_eq!(
            linear_regression(&bars, 4),
            linear_regression_store(&store, 4, &mut HashMap::new())
        );
        assert_vec_eq!(
            stddev(&bars, 4),
            stddev_store(&store, 4, &mut HashMap::new())
        );
        assert_vec_eq!(trix(&bars, 3), &trix_store(&store, 3, &mut HashMap::new()));
        assert_vec_eq!(
            tsi(&bars, 4, 2),
            tsi_store(&store, 4, 2, &mut HashMap::new())
        );
        assert_vec_eq!(
            momentum(&bars, 3),
            momentum_store(&store, 3, &mut HashMap::new())
        );
    }

    #[test]
    fn store_remaining_batch_matches_row_versions() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 9.5, 12.0),
            (14.0, 10.0, 13.0),
            (15.0, 11.0, 14.0),
            (16.0, 12.0, 15.0),
            (17.0, 13.0, 16.0),
        ]);
        for (bar, volume) in bars.iter_mut().zip(1..=8) {
            bar.volume = volume as f64;
        }
        let store = store_from_bars(bars.clone());

        for (row, store_output) in [
            (
                donchian(&bars, 3, &mut HashMap::new()),
                donchian_store(&store, 3, &mut HashMap::new()),
            ),
            (
                parabolic_sar(&bars, 0.02, 0.2, &mut HashMap::new()),
                parabolic_sar_store(&store, 0.02, 0.2, &mut HashMap::new()),
            ),
            (
                ichimoku(&bars, 3, 4, 5, &mut HashMap::new()),
                ichimoku_store(&store, 3, 4, 5, &mut HashMap::new()),
            ),
            (
                pivot_points(&bars, &mut HashMap::new()),
                pivot_points_store(&store, &mut HashMap::new()),
            ),
            (
                aroon(&bars, 3, &mut HashMap::new()),
                aroon_store(&store, 3, &mut HashMap::new()),
            ),
            (
                stoch_rsi(&bars, 3, 3, 2, 2, &mut HashMap::new()),
                stoch_rsi_store(&store, 3, 3, 2, 2, &mut HashMap::new()),
            ),
        ] {
            for row_output in &row {
                let store_vals = store_output
                    .iter()
                    .find(|o| o.name == row_output.name)
                    .map(|o| &o.values)
                    .unwrap();
                assert_series_eq(&row_output.values, store_vals);
            }
        }

        assert_vec_eq!(
            ultimate_oscillator(&bars, 2, 3, 4),
            ultimate_oscillator_store(&store, 2, 3, 4, &mut HashMap::new())
        );
        assert_vec_eq!(
            *chaikin_volatility_store(&store, 3, &mut HashMap::new()),
            chaikin_volatility_store(&store, 3, &mut HashMap::new())
        );
    }

    #[test]
    fn store_final_batch_matches_row_versions() {
        let mut bars = ohlc(&[
            (10.0, 8.0, 9.0),
            (11.0, 9.0, 10.0),
            (12.0, 10.0, 11.0),
            (13.0, 11.0, 12.0),
            (14.0, 12.0, 13.0),
            (15.0, 13.0, 14.0),
            (16.0, 14.0, 15.0),
            (17.0, 15.0, 16.0),
            (18.0, 16.0, 17.0),
            (19.0, 17.0, 18.0),
            (20.0, 18.0, 19.0),
            (21.0, 19.0, 20.0),
            (22.0, 20.0, 21.0),
            (23.0, 21.0, 22.0),
            (24.0, 22.0, 23.0),
            (25.0, 23.0, 24.0),
            (26.0, 24.0, 25.0),
            (27.0, 25.0, 26.0),
            (28.0, 26.0, 27.0),
            (29.0, 27.0, 28.0),
            (30.0, 28.0, 29.0),
            (31.0, 29.0, 30.0),
            (32.0, 30.0, 31.0),
            (33.0, 31.0, 32.0),
            (34.0, 32.0, 33.0),
            (35.0, 33.0, 34.0),
            (36.0, 34.0, 35.0),
            (37.0, 35.0, 36.0),
            (38.0, 36.0, 37.0),
            (39.0, 37.0, 38.0),
            (40.0, 38.0, 39.0),
        ]);
        for (i, bar) in bars.iter_mut().enumerate() {
            bar.volume = (i + 1) as f64;
        }
        let store = store_from_bars(bars.clone());

        assert_vec_eq!(dema(&bars, 5), &dema_store(&store, 5, &mut HashMap::new()));
        assert_vec_eq!(tema(&bars, 5), &tema_store(&store, 5, &mut HashMap::new()));
        assert_vec_eq!(
            trima(&bars, 5),
            &trima_store(&store, 5, &mut HashMap::new())
        );
        assert_vec_eq!(kst(&bars), &kst_store(&store, &mut HashMap::new()));
        assert_vec_eq!(bop(&bars), &bop_store(&store, &mut HashMap::new()));
        assert_eq!(
            chaikin_oscillator_store(
                &store,
                MacdParams {
                    fast: 3,
                    slow: 10,
                    signal: 9,
                },
                &mut HashMap::new()
            ),
            chaikin_oscillator_store(
                &store,
                MacdParams {
                    fast: 3,
                    slow: 10,
                    signal: 9,
                },
                &mut HashMap::new(),
            )
        );
        let row_envelope = envelope(&bars, 5, 2.0, &mut HashMap::new());
        let store_envelope = envelope_store(&store, 5, 2.0, &mut HashMap::new());
        assert_outputs_eq(
            &row_envelope,
            &store_envelope,
            &["upper", "middle", "lower"],
        );
    }

    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    fn latest_indicator_values_fast_reuses_visible_output_scratch() {
        let mut engine = ChartEngine::new();
        engine.indicators.push(Indicator {
            id: 7,
            kind: "RSI".to_string(),
            period: 0,
            stoch_period: 0,
            smooth: 0,
            signal: 0,
            tenkan_period: 0,
            kijun_period: 0,
            senkou_b_period: 0,
            macd: None,
            multiplier: 0.0,
            psar_step: 0.0,
            psar_max_step: 0.0,
            anchor: 0,
            outputs: IndicatorArena::from_outputs(vec![
                IndicatorOutput {
                    name: "value".to_string(),
                    values: vec![1.25],
                },
                IndicatorOutput {
                    name: "avg_gain".to_string(),
                    values: vec![f64::NAN],
                },
                IndicatorOutput {
                    name: "avg_loss".to_string(),
                    values: vec![-0.5],
                },
            ]),
        });

        let values = engine.latest_indicator_values_slice(7).unwrap();

        assert_eq!(values.len(), 1);
        assert_eq!(values[0], 1.25);
        assert_eq!(engine.latest_values_scratch.len(), 1);
    }

    #[test]
    fn indicator_output_values_fast_reuses_visible_output_scratch() {
        let mut engine = ChartEngine::new();
        engine.indicators.push(Indicator {
            id: 9,
            kind: "MACD".to_string(),
            period: 0,
            stoch_period: 0,
            smooth: 0,
            signal: 0,
            tenkan_period: 0,
            kijun_period: 0,
            senkou_b_period: 0,
            macd: None,
            multiplier: 0.0,
            psar_step: 0.0,
            psar_max_step: 0.0,
            anchor: 0,
            outputs: IndicatorArena::from_outputs(vec![
                IndicatorOutput {
                    name: "macd".to_string(),
                    values: vec![1.0, f64::NAN],
                },
                IndicatorOutput {
                    name: "signal".to_string(),
                    values: vec![2.0, 3.0],
                },
                IndicatorOutput {
                    name: "histogram".to_string(),
                    values: vec![-1.0, 0.5],
                },
                IndicatorOutput {
                    name: "fast_ema".to_string(),
                    values: vec![99.0, 100.0],
                },
            ]),
        });

        engine.populate_indicator_output_values_scratch(9).unwrap();

        assert_eq!(engine.indicator_values_scratch.len(), 3);
        assert_eq!(engine.indicator_values_scratch[0][0], 1.0);
        assert!(engine.indicator_values_scratch[0][1].is_nan());
        assert_eq!(engine.indicator_values_scratch[1].as_slice(), &[2.0, 3.0]);
        assert_eq!(engine.indicator_values_scratch[2].as_slice(), &[-1.0, 0.5]);
    }

    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    fn ema_nodes_are_reused_by_macd() {
        let bars = bars(&(1..=30).map(|value| value as f64).collect::<Vec<_>>());
        let store = store_from_bars(bars);
        let mut nodes: NodeCache = HashMap::new();

        let ema12 = compute_indicator_store(
            &store, "EMA", 12, 0, 0, 0, 9, 26, 52, None, 2.0, 0.02, 0.2, 0, &mut nodes,
        )[0]
        .values
        .clone();
        let macd = compute_indicator_store(
            &store,
            "MACD",
            0,
            0,
            0,
            0,
            9,
            26,
            52,
            Some(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            2.0,
            0.02,
            0.2,
            0,
            &mut nodes,
        );

        assert!(nodes.len() >= 2);
        assert_vec_eq!(nodes["ema:close:12"], ema12);
        assert_eq!(
            macd[0].values[29],
            nodes["ema:close:12"][29] - nodes["ema:close:26"][29]
        );
    }

    #[test]
    fn rsi_nodes_are_reused_by_stoch_rsi() {
        let bars = bars(&[1.0, 2.0, 1.0, 3.0, 2.0, 4.0, 3.0, 5.0]);
        let mut nodes = HashMap::new();

        let rsi = rsi_close(&bars, 3, &mut nodes);
        let stoch_rsi_outputs = stoch_rsi(&bars, 3, 3, 2, 2, &mut nodes);

        assert_vec_eq!(nodes["rsi:close:3"], rsi);
        assert_vec_eq!(nodes["stoch:rsi:3:3:2:2"], stoch_rsi_outputs[0].values);
    }

    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    fn wma_matches_latest_value() {
        let bars = bars(&[1.0, 2.0, 3.0, 4.0]);
        assert_eq!(
            latest_wma(&bars, 3),
            wma(&bars, 3).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn hma_matches_latest_value() {
        let bars = bars(&(1..=10).map(|value| value as f64).collect::<Vec<_>>());
        let outputs = hma(&bars, 4, &mut HashMap::new());
        assert_eq!(
            latest_hma(&bars, 4),
            outputs.last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    #[test]
    fn dema_matches_latest_value() {
        let bars = bars(&(1..=20).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(
            latest_dema(&bars, 5),
            dema(&bars, 5).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn tema_matches_latest_value() {
        let bars = bars(&(1..=20).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(
            latest_tema(&bars, 5),
            tema(&bars, 5).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    fn trima_matches_latest_value() {
        let bars = bars(&(1..=20).map(|value| value as f64).collect::<Vec<_>>());
        assert_eq!(
            latest_trima(&bars, 5),
            trima(&bars, 5).last().copied().and_then(nan_to_none)
        );
    }

    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    #[test]
    fn remove_indicator_reports_if_it_removed_one() {
        let mut engine = ChartEngine::new();
        let id = engine
            .add_indicator_from_config(IndicatorConfig {
                kind: "SMA".to_string(),
                period: Some(2),
                stoch_period: None,
                smooth: None,
                fast: None,
                slow: None,
                signal: None,
                multiplier: None,
                tenkan_period: None,
                kijun_period: None,
                senkou_b_period: None,
                psar_step: None,
                psar_max_step: None,
                anchor: None,
            })
            .unwrap();
        assert!(engine.remove_indicator(id));
        assert!(!engine.remove_indicator(id));
    }

    #[test]
    #[test]
    fn upsert_bar_replaces_latest_or_appends_next() {
        let mut bars = bars(&[1.0, 2.0]);
        upsert_bar(
            &mut bars,
            Bar {
                time: 1,
                open: 3.0,
                high: 3.0,
                low: 3.0,
                close: 3.0,
                volume: 1.0,
            },
        );
        upsert_bar(
            &mut bars,
            Bar {
                time: 2,
                open: 4.0,
                high: 4.0,
                low: 4.0,
                close: 4.0,
                volume: 1.0,
            },
        );

        assert_eq!(bars.len(), 3);
        assert_eq!(bars[1].close, 3.0);
        assert_eq!(bars[2].close, 4.0);
    }
}
