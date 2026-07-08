use crate::bar::CandleStore;
use crate::helpers::{upsert_output, IntoIndicatorOutputs};
use crate::indicators::*;
use crate::series::NodeCache;
use crate::types::{Indicator, IndicatorKind, IndicatorOutput, MacdParams};

use crate::indicators::adl::adl_store;
use crate::indicators::adx::adx_store;
use crate::indicators::alligator::alligator_store;
use crate::indicators::anchored_vwap::anchored_vwap_store;
use crate::indicators::aroon::aroon_store;
use crate::indicators::atr::atr_store;
use crate::indicators::atr_bands::atr_bands_store;
use crate::indicators::awesome_oscillator::awesome_oscillator_store;
use crate::indicators::beta::beta_store;
use crate::indicators::bollinger::bollinger_store;
use crate::indicators::bollinger_bandwidth::bollinger_bandwidth_store;
use crate::indicators::bollinger_pct_b::bollinger_pct_b_store;
use crate::indicators::bop::bop_store;
use crate::indicators::cci::cci_store;
use crate::indicators::center_of_gravity::center_of_gravity_store;
use crate::indicators::chaikin::{chaikin_oscillator_store, chaikin_volatility_store};
use crate::indicators::chande_forecast::chande_forecast_store;
use crate::indicators::chande_momentum::chande_momentum_store;
use crate::indicators::choppiness_index::choppiness_index_store;
use crate::indicators::cmf::cmf_store;
use crate::indicators::coppock_curve::coppock_curve_store;
use crate::indicators::correlation_coefficient::correlation_coefficient_store;
use crate::indicators::darvas_box::darvas_box_store;
use crate::indicators::dema::dema_store;
use crate::indicators::disparity_index::disparity_index_store;
use crate::indicators::donchian::{donchian_store, price_channel_store};
use crate::indicators::donchian_width::donchian_width_store;
use crate::indicators::dpo::dpo_store;
use crate::indicators::ease_of_movement::ease_of_movement_store;
use crate::indicators::ehler_fisher::ehler_fisher_store;
use crate::indicators::elder_impulse::elder_impulse_store;
use crate::indicators::elder_ray::elder_ray_store;
use crate::indicators::ema::ema_close_store;
use crate::indicators::envelope::envelope_store;
use crate::indicators::force_index::force_index_store;
use crate::indicators::fractal_chaos_bands::fractal_chaos_bands_store;
use crate::indicators::fractal_chaos_oscillator::fractal_chaos_oscillator_store;
use crate::indicators::gator_oscillator::gator_oscillator_store;
use crate::indicators::gmma::gmma_store;
use crate::indicators::gonogo_trend::gonogo_trend_store;
use crate::indicators::gopalakrishnan_range::gopalakrishnan_range_store;
use crate::indicators::high_low_bands::high_low_bands_store;
use crate::indicators::high_minus_low::high_minus_low_store;
use crate::indicators::highest_high::highest_high_store;
use crate::indicators::historical_volatility::historical_volatility_store;
use crate::indicators::hma::hma_store;
use crate::indicators::ichimoku::ichimoku_store;
use crate::indicators::intraday_momentum::intraday_momentum_store;
use crate::indicators::keltner::{keltner_store, starc_store};
use crate::indicators::klinger_volume::klinger_volume_store;
use crate::indicators::kst::kst_store;
use crate::indicators::linear_reg_forecast::linear_reg_forecast_store;
use crate::indicators::linear_reg_intercept::linear_reg_intercept_store;
use crate::indicators::linear_reg_r2::linear_reg_r2_store;
use crate::indicators::linear_reg_slope::linear_reg_slope_store;
use crate::indicators::linear_regression::linear_regression_store;
use crate::indicators::lowest_low::lowest_low_store;
use crate::indicators::ma_cross::ma_cross_store;
use crate::indicators::ma_deviation::ma_deviation_store;
use crate::indicators::macd::{macd_store, ppo_store};
use crate::indicators::market_facilitation::market_facilitation_store;
use crate::indicators::mass_index::mass_index_store;
use crate::indicators::median_price::median_price_store;
use crate::indicators::mfi::mfi_store;
use crate::indicators::momentum::momentum_store;
use crate::indicators::negative_volume_index::negative_volume_index_store;
use crate::indicators::obv::obv_store;
use crate::indicators::parabolic_sar::parabolic_sar_store;
use crate::indicators::performance_index::performance_index_store;
use crate::indicators::pivot_points::pivot_points_store;
use crate::indicators::positive_volume_index::positive_volume_index_store;
use crate::indicators::pretty_good_oscillator::pretty_good_oscillator_store;
use crate::indicators::price_momentum_oscillator::price_momentum_oscillator_store;
use crate::indicators::price_oscillator::price_oscillator_store;
use crate::indicators::price_relative::price_relative_store;
use crate::indicators::price_volume_trend::price_volume_trend_store;
use crate::indicators::prime_number_bands::prime_number_bands_store;
use crate::indicators::prime_number_oscillator::prime_number_oscillator_store;
use crate::indicators::projected_aggregate_volume::projected_aggregate_volume_store;
use crate::indicators::projected_volume_at_time::projected_volume_at_time_store;
use crate::indicators::psychological_line::psychological_line_store;
use crate::indicators::qstick::qstick_store;
use crate::indicators::rainbow_ma::rainbow_ma_store;
use crate::indicators::rainbow_oscillator::rainbow_oscillator_store;
use crate::indicators::random_walk_index::random_walk_index_store;
use crate::indicators::ravi::ravi_store;
use crate::indicators::relative_vigor::relative_vigor_store;
use crate::indicators::relative_volatility::relative_volatility_store;
use crate::indicators::roc::roc_store;
use crate::indicators::rsi::rsi_outputs_store;
use crate::indicators::schaff_trend_cycle::schaff_trend_cycle_store;
use crate::indicators::shinohara_intensity::shinohara_intensity_store;
use crate::indicators::sma::sma_close_store;
use crate::indicators::stddev::stddev_store;
use crate::indicators::stoch::stochastic_store;
use crate::indicators::stoch_rsi::stoch_rsi_store;
use crate::indicators::stochastic_momentum::stochastic_momentum_store;
use crate::indicators::supertrend::supertrend_store;
use crate::indicators::swing_index::swing_index_store;
use crate::indicators::tema::tema_store;
use crate::indicators::trade_volume_index::trade_volume_index_store;
use crate::indicators::trend_intensity::trend_intensity_store;
use crate::indicators::trima::trima_store;
use crate::indicators::trix::trix_store;
use crate::indicators::true_range::true_range_series_store;
use crate::indicators::tsi::tsi_store;
use crate::indicators::twiggs_money_flow::twiggs_money_flow_store;
use crate::indicators::typical_price::typical_price_store;
use crate::indicators::ulcer_index::ulcer_index_store;
use crate::indicators::ultimate_oscillator::ultimate_oscillator_store;
use crate::indicators::valuation_lines::valuation_lines_store;
use crate::indicators::vertical_horizontal_filter::vertical_horizontal_filter_store;
use crate::indicators::volume_chart::volume_chart_store;
use crate::indicators::volume_oscillator::volume_oscillator_store;
use crate::indicators::volume_profile::volume_profile_store;
use crate::indicators::volume_roc::volume_roc_store;
use crate::indicators::volume_underlay::volume_underlay_store;
use crate::indicators::vortex_indicator::vortex_indicator_store;
use crate::indicators::vwap::vwap_store;
use crate::indicators::vwma::vwma_store;
use crate::indicators::weighted_close::weighted_close_store;
use crate::indicators::williams_ad::williams_ad_store;
use crate::indicators::williams_r::williams_r_store;
use crate::indicators::wma::wma_store;
use crate::indicators::zigzag::zigzag_store;

#[allow(clippy::too_many_arguments)]
pub(crate) fn compute_indicator_store(
    store: &CandleStore,
    kind: IndicatorKind,
    period: usize,
    stoch_period: usize,
    smooth: usize,
    signal: usize,
    tenkan_period: usize,
    kijun_period: usize,
    senkou_b_period: usize,
    macd_params: Option<MacdParams>,
    multiplier: f64,
    psar_step: f64,
    psar_max_step: f64,
    anchor: usize,
    nodes: &mut NodeCache,
) -> Vec<IndicatorOutput> {
    match kind {
        IndicatorKind::SMA => sma_close_store(store, period, nodes).into_outputs(),
        IndicatorKind::EMA => ema_close_store(store, period, nodes).into_outputs(),
        IndicatorKind::RSI => rsi_outputs_store(store, period, nodes).into_outputs(),
        IndicatorKind::ROC => roc_store(store, period, nodes).into_outputs(),
        IndicatorKind::CCI => cci_store(store, period, nodes).into_outputs(),
        IndicatorKind::MFI => mfi_store(store, period, nodes).into_outputs(),
        IndicatorKind::CMF => cmf_store(store, period, nodes).into_outputs(),
        IndicatorKind::WILLIAMS_R => williams_r_store(store, period, nodes).into_outputs(),
        IndicatorKind::OBV => obv_store(store, nodes).into_outputs(),
        IndicatorKind::ADL => adl_store(store, nodes).into_outputs(),
        IndicatorKind::VWAP => vwap_store(store, nodes).into_outputs(),
        IndicatorKind::VWMA => vwma_store(store, period, nodes).into_outputs(),
        IndicatorKind::WILLIAMS_AD => williams_ad_store(store, nodes).into_outputs(),
        IndicatorKind::ATR => atr_store(store, period, nodes).into_outputs(),
        IndicatorKind::ADX => adx_store(store, period, nodes).into_outputs(),
        IndicatorKind::SUPERTREND => {
            supertrend_store(store, period, multiplier, nodes).into_outputs()
        }
        IndicatorKind::KELTNER => keltner_store(store, period, multiplier, nodes).into_outputs(),
        IndicatorKind::STARC => starc_store(store, period, multiplier, nodes).into_outputs(),
        IndicatorKind::WMA => wma_store(store, period, nodes).into_outputs(),
        IndicatorKind::HMA => hma_store(store, period, nodes).into_outputs(),
        IndicatorKind::LINEAR_REGRESSION => {
            linear_regression_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::LINEAR_REG_FORECAST => {
            linear_reg_forecast_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::LINEAR_REG_INTERCEPT => {
            linear_reg_intercept_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::DEMA => dema_store(store, period, nodes).into_outputs(),
        IndicatorKind::TEMA => tema_store(store, period, nodes).into_outputs(),
        IndicatorKind::TRIMA => trima_store(store, period, nodes).into_outputs(),
        IndicatorKind::STDDEV => stddev_store(store, period, nodes).into_outputs(),
        IndicatorKind::ENVELOPE => envelope_store(store, period, multiplier, nodes).into_outputs(),
        IndicatorKind::TRIX => trix_store(store, period, nodes).into_outputs(),
        IndicatorKind::TSI => tsi_store(store, period, stoch_period, nodes).into_outputs(),
        IndicatorKind::KST => kst_store(store, nodes).into_outputs(),
        IndicatorKind::BOP => bop_store(store, nodes).into_outputs(),
        IndicatorKind::MOMENTUM => momentum_store(store, period, nodes).into_outputs(),
        IndicatorKind::DPO => dpo_store(store, period, nodes).into_outputs(),
        IndicatorKind::FORCE_INDEX => force_index_store(store, period, nodes).into_outputs(),
        IndicatorKind::PRICE_CHANNEL => price_channel_store(store, period, nodes).into_outputs(),
        IndicatorKind::STOCHASTIC => stochastic_store(store, period, smooth, nodes).into_outputs(),
        IndicatorKind::BB => bollinger_store(store, period, multiplier, nodes).into_outputs(),
        IndicatorKind::DONCHIAN => donchian_store(store, period, nodes).into_outputs(),
        IndicatorKind::PARABOLIC_SAR => {
            parabolic_sar_store(store, psar_step, psar_max_step, nodes).into_outputs()
        }
        IndicatorKind::ICHIMOKU => {
            ichimoku_store(store, tenkan_period, kijun_period, senkou_b_period, nodes)
                .into_outputs()
        }
        IndicatorKind::PIVOT_POINTS => pivot_points_store(store, nodes).into_outputs(),
        IndicatorKind::AROON => aroon_store(store, period, nodes).into_outputs(),
        IndicatorKind::ULTIMATE_OSCILLATOR => {
            ultimate_oscillator_store(store, period, stoch_period, smooth, nodes).into_outputs()
        }
        IndicatorKind::CHAIKIN_VOLATILITY => {
            chaikin_volatility_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::STOCH_RSI => {
            stoch_rsi_store(store, period, stoch_period, smooth, signal, nodes).into_outputs()
        }
        IndicatorKind::CHAIKIN_OSCILLATOR => chaikin_oscillator_store(
            store,
            macd_params.unwrap_or(MacdParams {
                fast: 3,
                slow: 10,
                signal: 9,
            }),
            nodes,
        )
        .into_outputs(),
        IndicatorKind::MACD => macd_store(
            store,
            macd_params.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            nodes,
        )
        .into_outputs(),
        IndicatorKind::PPO => ppo_store(
            store,
            macd_params.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            nodes,
        )
        .into_outputs(),
        IndicatorKind::MEDIAN_PRICE => median_price_store(store, nodes).into_outputs(),
        IndicatorKind::HIGHEST_HIGH => highest_high_store(store, period, nodes).into_outputs(),
        IndicatorKind::LOWEST_LOW => lowest_low_store(store, period, nodes).into_outputs(),
        IndicatorKind::ALLIGATOR => alligator_store(store, nodes).into_outputs(),
        IndicatorKind::ATR_BANDS => {
            atr_bands_store(store, period, multiplier, nodes).into_outputs()
        }
        IndicatorKind::HIGH_LOW_BANDS => high_low_bands_store(store, period, nodes).into_outputs(),
        IndicatorKind::FRACTAL_CHAOS_BANDS => {
            fractal_chaos_bands_store(store, nodes).into_outputs()
        }
        IndicatorKind::GMMA => gmma_store(store, nodes).into_outputs(),
        IndicatorKind::ANCHORED_VWAP => anchored_vwap_store(store, anchor, nodes).into_outputs(),
        IndicatorKind::TYPICAL_PRICE => typical_price_store(store, nodes).into_outputs(),
        IndicatorKind::WEIGHTED_CLOSE => weighted_close_store(store, nodes).into_outputs(),
        IndicatorKind::MA_CROSS => ma_cross_store(
            store,
            macd_params.map_or(period, |m| m.fast),
            macd_params.map_or(stoch_period, |m| m.slow),
            nodes,
        )
        .into_outputs(),
        IndicatorKind::RAINBOW_MA => rainbow_ma_store(store, period, nodes).into_outputs(),
        IndicatorKind::PRIME_NUMBER_BANDS => prime_number_bands_store(store, nodes).into_outputs(),
        IndicatorKind::TIME_SERIES_FORECAST => {
            linear_reg_forecast_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::VALUATION_LINES => {
            valuation_lines_store(store, period, multiplier, nodes).into_outputs()
        }
        IndicatorKind::BETA => beta_store(store, period, nodes).into_outputs(),
        IndicatorKind::CORRELATION_COEFFICIENT => {
            correlation_coefficient_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::PERFORMANCE_INDEX => performance_index_store(store, nodes).into_outputs(),
        IndicatorKind::PRICE_RELATIVE => price_relative_store(store, period, nodes).into_outputs(),
        IndicatorKind::AWESOME_OSCILLATOR => awesome_oscillator_store(store, nodes).into_outputs(),
        IndicatorKind::BOLLINGER_PCT_B => {
            bollinger_pct_b_store(store, period, multiplier, nodes).into_outputs()
        }
        IndicatorKind::CENTER_OF_GRAVITY => {
            center_of_gravity_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::CHANDE_FORECAST => {
            chande_forecast_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::CHANDE_MOMENTUM => {
            chande_momentum_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::COPPOCK_CURVE => coppock_curve_store(store, nodes).into_outputs(),
        IndicatorKind::DISPARITY_INDEX => {
            disparity_index_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::EASE_OF_MOVEMENT => {
            ease_of_movement_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::EHLER_FISHER => ehler_fisher_store(store, period, nodes).into_outputs(),
        IndicatorKind::ELDER_RAY => elder_ray_store(store, period, nodes).into_outputs(),
        IndicatorKind::FRACTAL_CHAOS_OSCILLATOR => {
            fractal_chaos_oscillator_store(store, nodes).into_outputs()
        }
        IndicatorKind::GATOR_OSCILLATOR => gator_oscillator_store(store, nodes).into_outputs(),
        IndicatorKind::INTRADAY_MOMENTUM => {
            intraday_momentum_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::LINEAR_REG_SLOPE => {
            linear_reg_slope_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::MA_DEVIATION => ma_deviation_store(store, period, nodes).into_outputs(),
        IndicatorKind::PRETTY_GOOD_OSCILLATOR => {
            pretty_good_oscillator_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::PRICE_MOMENTUM_OSCILLATOR => {
            price_momentum_oscillator_store(store, period, smooth, nodes).into_outputs()
        }
        IndicatorKind::PRICE_OSCILLATOR => price_oscillator_store(
            store,
            macd_params.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            }),
            nodes,
        )
        .into_outputs(),
        IndicatorKind::RAINBOW_OSCILLATOR => {
            rainbow_oscillator_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::RAVI => ravi_store(store, period, stoch_period, nodes).into_outputs(),
        IndicatorKind::RELATIVE_VIGOR => relative_vigor_store(store, period, nodes).into_outputs(),
        IndicatorKind::SCHAFF_TREND_CYCLE => schaff_trend_cycle_store(
            store,
            macd_params.map_or(12, |m| m.fast),
            macd_params.map_or(26, |m| m.slow),
            stoch_period,
            nodes,
        )
        .into_outputs(),
        IndicatorKind::STOCHASTIC_MOMENTUM => {
            stochastic_momentum_store(store, period, smooth, nodes).into_outputs()
        }
        IndicatorKind::SWING_INDEX => swing_index_store(store, nodes).into_outputs(),
        IndicatorKind::TREND_INTENSITY => {
            trend_intensity_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::VOLUME_OSCILLATOR => volume_oscillator_store(
            store,
            macd_params.unwrap_or(MacdParams {
                fast: 5,
                slow: 10,
                signal: 9,
            }),
            nodes,
        )
        .into_outputs(),
        IndicatorKind::KLINGER_VOLUME => klinger_volume_store(store, nodes).into_outputs(),
        IndicatorKind::MARKET_FACILITATION => {
            market_facilitation_store(store, nodes).into_outputs()
        }
        IndicatorKind::NEGATIVE_VOLUME_INDEX => {
            negative_volume_index_store(store, nodes).into_outputs()
        }
        IndicatorKind::POSITIVE_VOLUME_INDEX => {
            positive_volume_index_store(store, nodes).into_outputs()
        }
        IndicatorKind::PRICE_VOLUME_TREND => price_volume_trend_store(store, nodes).into_outputs(),
        IndicatorKind::TRADE_VOLUME_INDEX => trade_volume_index_store(store, nodes).into_outputs(),
        IndicatorKind::TWIGGS_MONEY_FLOW => {
            twiggs_money_flow_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::PROJECTED_AGGREGATE_VOLUME => {
            projected_aggregate_volume_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::PROJECTED_VOLUME_AT_TIME => {
            projected_volume_at_time_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::HISTORICAL_VOLATILITY => {
            historical_volatility_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::LINEAR_REG_R2 => linear_reg_r2_store(store, period, nodes).into_outputs(),
        IndicatorKind::PRIME_NUMBER_OSCILLATOR => {
            prime_number_oscillator_store(store, nodes).into_outputs()
        }
        IndicatorKind::RANDOM_WALK_INDEX => {
            random_walk_index_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::DARVAS_BOX => darvas_box_store(store, nodes).into_outputs(),
        IndicatorKind::VOLUME_PROFILE => volume_profile_store(store, period, nodes).into_outputs(),
        IndicatorKind::CHOPPINESS_INDEX => {
            choppiness_index_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::ELDER_IMPULSE => elder_impulse_store(store, period, nodes).into_outputs(),
        IndicatorKind::GONOGO_TREND => gonogo_trend_store(store, period, nodes).into_outputs(),
        IndicatorKind::PSYCHOLOGICAL_LINE => {
            psychological_line_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::QSTICK => qstick_store(store, period, nodes).into_outputs(),
        IndicatorKind::SHINOHARA_INTENSITY => {
            shinohara_intensity_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::ULCER_INDEX => ulcer_index_store(store, period, nodes).into_outputs(),
        IndicatorKind::VERTICAL_HORIZONTAL_FILTER => {
            vertical_horizontal_filter_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::VORTEX_INDICATOR => {
            vortex_indicator_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::ZIGZAG => zigzag_store(store, multiplier, nodes).into_outputs(),
        IndicatorKind::BOLLINGER_BANDWIDTH => {
            bollinger_bandwidth_store(store, period, multiplier, nodes).into_outputs()
        }
        IndicatorKind::DONCHIAN_WIDTH => donchian_width_store(store, period, nodes).into_outputs(),
        IndicatorKind::GOPALAKRISHNAN_RANGE => {
            gopalakrishnan_range_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::HIGH_MINUS_LOW => high_minus_low_store(store, nodes).into_outputs(),
        IndicatorKind::MASS_INDEX => mass_index_store(store, period, nodes).into_outputs(),
        IndicatorKind::RELATIVE_VOLATILITY => {
            relative_volatility_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::TRUE_RANGE => true_range_series_store(store, nodes).into_outputs(),
        IndicatorKind::VOLUME_CHART => volume_chart_store(store, nodes).into_outputs(),
        IndicatorKind::VOLUME_ROC => volume_roc_store(store, period, nodes).into_outputs(),
        IndicatorKind::VOLUME_UNDERLAY => volume_underlay_store(store, nodes).into_outputs(),
    }
}

pub(crate) fn update_indicator_incremental(
    store: &CandleStore,
    indicator: &mut Indicator,
    target_len: usize,
) {
    indicator.outputs.ensure_len(target_len);
    match indicator.kind {
        IndicatorKind::SMA => upsert_output(
            &mut indicator.outputs,
            "value",
            target_len,
            latest_sma_store(store, indicator.period),
        ),
        IndicatorKind::EMA => {
            let value = latest_ema_store(store, indicator.period, indicator.outputs.get_slot(0));
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::RSI => {
            let (value, avg_gain, avg_loss) =
                latest_rsi_store(store, indicator.period, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "avg_gain", target_len, avg_gain);
            upsert_output(&mut indicator.outputs, "avg_loss", target_len, avg_loss);
        }
        IndicatorKind::ROC => {
            let value = latest_roc_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::CCI => {
            let value = latest_cci_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::WILLIAMS_R => {
            let value = latest_williams_r_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::MFI => {
            let value = latest_mfi_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::CMF => {
            let value = latest_cmf_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::STOCH_RSI => {
            let (k, d) = latest_stoch_rsi_store(
                store,
                indicator.period,
                indicator.stoch_period,
                indicator.smooth,
                indicator.signal,
            );
            upsert_output(&mut indicator.outputs, "k", target_len, k);
            upsert_output(&mut indicator.outputs, "d", target_len, d);
        }
        IndicatorKind::OBV => {
            let value = latest_obv_store(store, indicator.outputs.get_slot(0));
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::ATR => {
            let value = latest_atr_store(store, indicator.period, indicator.outputs.get_slot(0));
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::ADX => {
            let (value, plus_di, minus_di, tr_avg, plus_dm_avg, minus_dm_avg, dx) =
                latest_adx_store(store, indicator.period, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "plus_di", target_len, plus_di);
            upsert_output(&mut indicator.outputs, "minus_di", target_len, minus_di);
            upsert_output(&mut indicator.outputs, "tr_avg", target_len, tr_avg);
            upsert_output(
                &mut indicator.outputs,
                "plus_dm_avg",
                target_len,
                plus_dm_avg,
            );
            upsert_output(
                &mut indicator.outputs,
                "minus_dm_avg",
                target_len,
                minus_dm_avg,
            );
            upsert_output(&mut indicator.outputs, "dx", target_len, dx);
        }
        IndicatorKind::SUPERTREND => {
            let (value, upper_band, lower_band, trend) = latest_supertrend_store(
                store,
                indicator.period,
                indicator.multiplier,
                &indicator.outputs,
            );
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "upper_band", target_len, upper_band);
            upsert_output(&mut indicator.outputs, "lower_band", target_len, lower_band);
            upsert_output(&mut indicator.outputs, "trend", target_len, trend);
        }
        IndicatorKind::KELTNER => {
            let (upper, middle, lower) = latest_keltner_store(
                store,
                indicator.period,
                indicator.multiplier,
                &indicator.outputs,
            );
            upsert_output(&mut indicator.outputs, "upper", target_len, upper);
            upsert_output(&mut indicator.outputs, "middle", target_len, middle);
            upsert_output(&mut indicator.outputs, "lower", target_len, lower);
        }
        IndicatorKind::DONCHIAN => {
            let (upper, middle, lower) = latest_donchian_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "upper", target_len, upper);
            upsert_output(&mut indicator.outputs, "middle", target_len, middle);
            upsert_output(&mut indicator.outputs, "lower", target_len, lower);
        }
        IndicatorKind::PARABOLIC_SAR => {
            let (value, ep, af, trend) = latest_parabolic_sar_store(
                store,
                indicator.psar_step,
                indicator.psar_max_step,
                &indicator.outputs,
            );
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "ep", target_len, ep);
            upsert_output(&mut indicator.outputs, "af", target_len, af);
            upsert_output(&mut indicator.outputs, "trend", target_len, trend);
        }
        IndicatorKind::ICHIMOKU => {
            let (tenkan, kijun, senkou_a, senkou_b, chikou) = latest_ichimoku_store(
                store,
                indicator.tenkan_period,
                indicator.kijun_period,
                indicator.senkou_b_period,
            );
            upsert_output(&mut indicator.outputs, "tenkan", target_len, tenkan);
            upsert_output(&mut indicator.outputs, "kijun", target_len, kijun);
            upsert_output(&mut indicator.outputs, "senkou_a", target_len, senkou_a);
            upsert_output(&mut indicator.outputs, "senkou_b", target_len, senkou_b);
            upsert_output(&mut indicator.outputs, "chikou", target_len, chikou);
        }
        IndicatorKind::PIVOT_POINTS => {
            let (pp, r1, s1, r2, s2) = latest_pivot_points_store(store);
            upsert_output(&mut indicator.outputs, "pp", target_len, pp);
            upsert_output(&mut indicator.outputs, "r1", target_len, r1);
            upsert_output(&mut indicator.outputs, "s1", target_len, s1);
            upsert_output(&mut indicator.outputs, "r2", target_len, r2);
            upsert_output(&mut indicator.outputs, "s2", target_len, s2);
        }
        IndicatorKind::AROON => {
            let (up, down, oscillator) = latest_aroon_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "up", target_len, up);
            upsert_output(&mut indicator.outputs, "down", target_len, down);
            upsert_output(&mut indicator.outputs, "oscillator", target_len, oscillator);
        }
        IndicatorKind::ADL => {
            let value = latest_adl_store(store, indicator.outputs.get_slot(0));
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::WMA => {
            let value = latest_wma_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::HMA => {
            let value = latest_hma_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::LINEAR_REGRESSION => {
            let value = latest_linear_regression_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::DEMA => {
            let (value, ema1, ema2) =
                latest_dema_store(store, indicator.period, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "ema1", target_len, ema1);
            upsert_output(&mut indicator.outputs, "ema2", target_len, ema2);
        }
        IndicatorKind::TEMA => {
            let (value, ema1, ema2, ema3) =
                latest_tema_store(store, indicator.period, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "ema1", target_len, ema1);
            upsert_output(&mut indicator.outputs, "ema2", target_len, ema2);
            upsert_output(&mut indicator.outputs, "ema3", target_len, ema3);
        }
        IndicatorKind::TRIMA => {
            let value = latest_trima_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::STDDEV => {
            let value = latest_stddev_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::ENVELOPE => {
            let (upper, middle, lower) =
                latest_envelope_store(store, indicator.period, indicator.multiplier);
            upsert_output(&mut indicator.outputs, "upper", target_len, upper);
            upsert_output(&mut indicator.outputs, "middle", target_len, middle);
            upsert_output(&mut indicator.outputs, "lower", target_len, lower);
        }
        IndicatorKind::TRIX => {
            let (value, ema1, ema2, ema3) =
                latest_trix_store(store, indicator.period, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "ema1", target_len, ema1);
            upsert_output(&mut indicator.outputs, "ema2", target_len, ema2);
            upsert_output(&mut indicator.outputs, "ema3", target_len, ema3);
        }
        IndicatorKind::TSI => {
            let (value, m_ema1, m_ema2, a_ema1, a_ema2) = latest_tsi_store(
                store,
                indicator.period,
                indicator.stoch_period,
                &indicator.outputs,
            );
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "m_ema1", target_len, m_ema1);
            upsert_output(&mut indicator.outputs, "m_ema2", target_len, m_ema2);
            upsert_output(&mut indicator.outputs, "a_ema1", target_len, a_ema1);
            upsert_output(&mut indicator.outputs, "a_ema2", target_len, a_ema2);
        }
        IndicatorKind::KST => {
            let value = latest_kst_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::BOP => {
            let value = latest_bop_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::DPO => {
            let value = latest_dpo_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::MOMENTUM => {
            let value = latest_momentum_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::ULTIMATE_OSCILLATOR => {
            let value = latest_ultimate_oscillator_store(
                store,
                indicator.period,
                indicator.stoch_period,
                indicator.smooth,
            );
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::CHAIKIN_OSCILLATOR => {
            let params = indicator.macd.unwrap_or(MacdParams {
                fast: 3,
                slow: 10,
                signal: 9,
            });
            let (value, adl, fast_ema, slow_ema) =
                latest_chaikin_oscillator_store(store, params, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "adl", target_len, adl);
            upsert_output(&mut indicator.outputs, "fast_ema", target_len, fast_ema);
            upsert_output(&mut indicator.outputs, "slow_ema", target_len, slow_ema);
        }
        IndicatorKind::FORCE_INDEX => {
            let (value, fi_ema) =
                latest_force_index_store(store, indicator.period, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "fi_ema", target_len, fi_ema);
        }
        IndicatorKind::VWMA => {
            let value = latest_vwma_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::WILLIAMS_AD => {
            let value = latest_williams_ad_store(store, indicator.outputs.get_slot(0));
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::CHAIKIN_VOLATILITY => {
            let (value, hl_ema) =
                latest_chaikin_volatility_store(store, indicator.period, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "hl_ema", target_len, hl_ema);
        }
        IndicatorKind::PRICE_CHANNEL => {
            let (upper, middle, lower) = latest_price_channel_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "upper", target_len, upper);
            upsert_output(&mut indicator.outputs, "middle", target_len, middle);
            upsert_output(&mut indicator.outputs, "lower", target_len, lower);
        }
        IndicatorKind::STARC => {
            let (upper, middle, lower) =
                latest_starc_store(store, indicator.period, indicator.multiplier);
            upsert_output(&mut indicator.outputs, "upper", target_len, upper);
            upsert_output(&mut indicator.outputs, "middle", target_len, middle);
            upsert_output(&mut indicator.outputs, "lower", target_len, lower);
        }
        IndicatorKind::VWAP => {
            let (value, cumulative_pv, cumulative_volume) =
                latest_vwap_store(store, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(
                &mut indicator.outputs,
                "cumulative_pv",
                target_len,
                cumulative_pv,
            );
            upsert_output(
                &mut indicator.outputs,
                "cumulative_volume",
                target_len,
                cumulative_volume,
            );
        }
        IndicatorKind::BB => {
            let (upper, middle, lower) =
                latest_bollinger_store(store, indicator.period, indicator.multiplier);
            upsert_output(&mut indicator.outputs, "upper", target_len, upper);
            upsert_output(&mut indicator.outputs, "middle", target_len, middle);
            upsert_output(&mut indicator.outputs, "lower", target_len, lower);
        }
        IndicatorKind::STOCHASTIC => {
            let (k, d) = latest_stochastic_store(
                store,
                indicator.period,
                indicator.smooth,
                &indicator.outputs,
            );
            upsert_output(&mut indicator.outputs, "k", target_len, k);
            upsert_output(&mut indicator.outputs, "d", target_len, d);
        }
        IndicatorKind::MACD => {
            let macd = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            let (macd_line, signal, histogram, fast_ema, slow_ema) =
                latest_macd_store(store, macd, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "macd", target_len, macd_line);
            upsert_output(&mut indicator.outputs, "signal", target_len, signal);
            upsert_output(&mut indicator.outputs, "histogram", target_len, histogram);
            upsert_output(&mut indicator.outputs, "fast_ema", target_len, fast_ema);
            upsert_output(&mut indicator.outputs, "slow_ema", target_len, slow_ema);
        }
        IndicatorKind::PPO => {
            let params = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            let (ppo, signal, histogram) = latest_ppo_store(store, params, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "ppo", target_len, ppo);
            upsert_output(&mut indicator.outputs, "signal", target_len, signal);
            upsert_output(&mut indicator.outputs, "histogram", target_len, histogram);
        }
        IndicatorKind::MEDIAN_PRICE => {
            let value = latest_median_price_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::HIGHEST_HIGH => {
            let value = latest_highest_high_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::LOWEST_LOW => {
            let value = latest_lowest_low_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::ALLIGATOR => {
            let (jaw, teeth, lips) = latest_alligator_store(store);
            upsert_output(&mut indicator.outputs, "jaw", target_len, jaw);
            upsert_output(&mut indicator.outputs, "teeth", target_len, teeth);
            upsert_output(&mut indicator.outputs, "lips", target_len, lips);
        }
        IndicatorKind::ATR_BANDS => {
            let (upper, middle, lower) = latest_atr_bands_store(
                store,
                indicator.period,
                indicator.multiplier,
                &indicator.outputs,
            );
            upsert_output(&mut indicator.outputs, "upper", target_len, upper);
            upsert_output(&mut indicator.outputs, "middle", target_len, middle);
            upsert_output(&mut indicator.outputs, "lower", target_len, lower);
        }
        IndicatorKind::HIGH_LOW_BANDS => {
            let (upper, middle, lower) = latest_high_low_bands_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "upper", target_len, upper);
            upsert_output(&mut indicator.outputs, "middle", target_len, middle);
            upsert_output(&mut indicator.outputs, "lower", target_len, lower);
        }
        IndicatorKind::FRACTAL_CHAOS_BANDS => {
            let (upper, lower) = latest_fractal_chaos_bands_store(store);
            upsert_output(&mut indicator.outputs, "upper", target_len, upper);
            upsert_output(&mut indicator.outputs, "lower", target_len, lower);
        }
        IndicatorKind::GMMA => {
            let results = latest_gmma_store(store, &indicator.outputs);
            for (name, value) in results {
                upsert_output(&mut indicator.outputs, &name, target_len, value);
            }
        }
        IndicatorKind::LINEAR_REG_FORECAST => {
            let value = latest_linear_reg_forecast_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::LINEAR_REG_INTERCEPT => {
            let value = latest_linear_reg_intercept_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::ANCHORED_VWAP => {
            let (value, cum_pv, cum_vol) =
                latest_anchored_vwap_store(store, indicator.anchor, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "cumulative_pv", target_len, cum_pv);
            upsert_output(
                &mut indicator.outputs,
                "cumulative_volume",
                target_len,
                cum_vol,
            );
        }
        IndicatorKind::TYPICAL_PRICE => {
            let value = latest_typical_price_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::WEIGHTED_CLOSE => {
            let value = latest_weighted_close_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::MA_CROSS => {
            let params = indicator.macd.unwrap_or(MacdParams {
                fast: 10,
                slow: 20,
                signal: 9,
            });
            let (fast, slow, histogram) = latest_ma_cross_store(store, params.fast, params.slow);
            upsert_output(&mut indicator.outputs, "fast", target_len, fast);
            upsert_output(&mut indicator.outputs, "slow", target_len, slow);
            upsert_output(&mut indicator.outputs, "histogram", target_len, histogram);
        }
        IndicatorKind::RAINBOW_MA => {
            let results = latest_rainbow_ma_store(store, indicator.period);
            for (name, value) in results {
                upsert_output(&mut indicator.outputs, &name, target_len, value);
            }
        }
        IndicatorKind::PRIME_NUMBER_BANDS => {
            let (upper, lower) = latest_prime_number_bands_store(store);
            upsert_output(&mut indicator.outputs, "upper", target_len, upper);
            upsert_output(&mut indicator.outputs, "lower", target_len, lower);
        }
        IndicatorKind::TIME_SERIES_FORECAST => {
            let value = latest_linear_reg_forecast_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::VALUATION_LINES => {
            let (upper, middle, lower) =
                latest_valuation_lines_store(store, indicator.period, indicator.multiplier);
            upsert_output(&mut indicator.outputs, "upper", target_len, upper);
            upsert_output(&mut indicator.outputs, "middle", target_len, middle);
            upsert_output(&mut indicator.outputs, "lower", target_len, lower);
        }
        IndicatorKind::BETA => {
            let value = latest_beta_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::CORRELATION_COEFFICIENT => {
            let value = latest_correlation_coefficient_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::PERFORMANCE_INDEX => {
            let value = latest_performance_index_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::PRICE_RELATIVE => {
            let value = latest_price_relative_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::AWESOME_OSCILLATOR => {
            let value = latest_awesome_oscillator_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::BOLLINGER_PCT_B => {
            let value = latest_bollinger_pct_b_store(store, indicator.period, indicator.multiplier);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::CENTER_OF_GRAVITY => {
            let value = latest_center_of_gravity_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::CHANDE_FORECAST => {
            let value = latest_chande_forecast_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::CHANDE_MOMENTUM => {
            let value = latest_chande_momentum_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::COPPOCK_CURVE => {
            let value = latest_coppock_curve_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::DISPARITY_INDEX => {
            let value = latest_disparity_index_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::EASE_OF_MOVEMENT => {
            let value = latest_ease_of_movement_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::EHLER_FISHER => {
            let (fisher, trigger) = latest_ehler_fisher_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "fisher", target_len, fisher);
            upsert_output(&mut indicator.outputs, "trigger", target_len, trigger);
        }
        IndicatorKind::ELDER_RAY => {
            let (bull, bear) = latest_elder_ray_store(store, indicator.period, &indicator.outputs);
            upsert_output(&mut indicator.outputs, "bull", target_len, bull);
            upsert_output(&mut indicator.outputs, "bear", target_len, bear);
        }
        IndicatorKind::FRACTAL_CHAOS_OSCILLATOR => {
            let value = latest_fractal_chaos_oscillator_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::GATOR_OSCILLATOR => {
            let (upper, lower) = latest_gator_oscillator_store(store);
            upsert_output(&mut indicator.outputs, "upper", target_len, upper);
            upsert_output(&mut indicator.outputs, "lower", target_len, lower);
        }
        IndicatorKind::INTRADAY_MOMENTUM => {
            let value = latest_intraday_momentum_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::LINEAR_REG_SLOPE => {
            let value = latest_linear_reg_slope_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::MA_DEVIATION => {
            let value = latest_ma_deviation_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::PRETTY_GOOD_OSCILLATOR => {
            let value = latest_pretty_good_oscillator_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::PRICE_MOMENTUM_OSCILLATOR => {
            let value =
                latest_price_momentum_oscillator_store(store, indicator.period, indicator.smooth);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::PRICE_OSCILLATOR => {
            let params = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            let value = latest_price_oscillator_store(store, params);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::RAINBOW_OSCILLATOR => {
            let value = latest_rainbow_oscillator_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::RAVI => {
            let value = latest_ravi_store(store, indicator.period, indicator.stoch_period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::RELATIVE_VIGOR => {
            let (value, signal) = latest_relative_vigor_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
            upsert_output(&mut indicator.outputs, "signal", target_len, signal);
        }
        IndicatorKind::SCHAFF_TREND_CYCLE => {
            let params = indicator.macd.unwrap_or(MacdParams {
                fast: 12,
                slow: 26,
                signal: 9,
            });
            let value = latest_schaff_trend_cycle_store(
                store,
                params.fast,
                params.slow,
                indicator.stoch_period,
            );
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::STOCHASTIC_MOMENTUM => {
            let value = latest_stochastic_momentum_store(store, indicator.period, indicator.smooth);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::SWING_INDEX => {
            let value = latest_swing_index_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::TREND_INTENSITY => {
            let value = latest_trend_intensity_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::VOLUME_OSCILLATOR => {
            let params = indicator.macd.unwrap_or(MacdParams {
                fast: 5,
                slow: 10,
                signal: 9,
            });
            let value = latest_volume_oscillator_store(store, params);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::KLINGER_VOLUME => {
            let value = latest_klinger_volume_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::MARKET_FACILITATION => {
            let value = latest_market_facilitation_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::NEGATIVE_VOLUME_INDEX => {
            let value = latest_negative_volume_index_store(store, indicator.outputs.get_slot(0));
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::POSITIVE_VOLUME_INDEX => {
            let value = latest_positive_volume_index_store(store, indicator.outputs.get_slot(0));
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::PRICE_VOLUME_TREND => {
            let value = latest_price_volume_trend_store(store, indicator.outputs.get_slot(0));
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::TRADE_VOLUME_INDEX => {
            let value = latest_trade_volume_index_store(store, indicator.outputs.get_slot(0));
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::TWIGGS_MONEY_FLOW => {
            let value = latest_twiggs_money_flow_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::PROJECTED_AGGREGATE_VOLUME => {
            let value = latest_projected_aggregate_volume_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::PROJECTED_VOLUME_AT_TIME => {
            let value = latest_projected_volume_at_time_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::HISTORICAL_VOLATILITY => {
            let value = latest_historical_volatility_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::LINEAR_REG_R2 => {
            let value = latest_linear_reg_r2_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::PRIME_NUMBER_OSCILLATOR => {
            let value = latest_prime_number_oscillator_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::RANDOM_WALK_INDEX => {
            let (high, low) = latest_random_walk_index_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "high", target_len, high);
            upsert_output(&mut indicator.outputs, "low", target_len, low);
        }
        IndicatorKind::DARVAS_BOX => {
            let (top, bottom) = latest_darvas_box_store(store);
            upsert_output(&mut indicator.outputs, "top", target_len, top);
            upsert_output(&mut indicator.outputs, "bottom", target_len, bottom);
        }
        IndicatorKind::VOLUME_PROFILE => {
            let (poc, vah, val) = latest_volume_profile_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "poc", target_len, poc);
            upsert_output(&mut indicator.outputs, "vah", target_len, vah);
            upsert_output(&mut indicator.outputs, "val", target_len, val);
        }
        IndicatorKind::CHOPPINESS_INDEX => {
            let value = latest_choppiness_index_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::ELDER_IMPULSE => {
            let value = latest_elder_impulse_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::GONOGO_TREND => {
            let value = latest_gonogo_trend_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::PSYCHOLOGICAL_LINE => {
            let value = latest_psychological_line_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::QSTICK => {
            let value = latest_qstick_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::SHINOHARA_INTENSITY => {
            let (strong, weak) = latest_shinohara_intensity_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "strong", target_len, strong);
            upsert_output(&mut indicator.outputs, "weak", target_len, weak);
        }
        IndicatorKind::ULCER_INDEX => {
            let value = latest_ulcer_index_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::VERTICAL_HORIZONTAL_FILTER => {
            let value = latest_vertical_horizontal_filter_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::VORTEX_INDICATOR => {
            let (plus, minus) = latest_vortex_indicator_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "plus", target_len, plus);
            upsert_output(&mut indicator.outputs, "minus", target_len, minus);
        }
        IndicatorKind::ZIGZAG => {
            let value = latest_zigzag_store(store, indicator.multiplier);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::BOLLINGER_BANDWIDTH => {
            let value =
                latest_bollinger_bandwidth_store(store, indicator.period, indicator.multiplier);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::DONCHIAN_WIDTH => {
            let value = latest_donchian_width_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::GOPALAKRISHNAN_RANGE => {
            let value = latest_gopalakrishnan_range_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::HIGH_MINUS_LOW => {
            let value = latest_high_minus_low_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::MASS_INDEX => {
            let value = latest_mass_index_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::RELATIVE_VOLATILITY => {
            let value = latest_relative_volatility_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::TRUE_RANGE => {
            let value = latest_true_range_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::VOLUME_CHART => {
            let value = latest_volume_chart_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::VOLUME_ROC => {
            let value = latest_volume_roc_store(store, indicator.period);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
        IndicatorKind::VOLUME_UNDERLAY => {
            let value = latest_volume_underlay_store(store);
            upsert_output(&mut indicator.outputs, "value", target_len, value);
        }
    }
}
