use crate::bar::CandleStore;
use crate::helpers::IntoIndicatorOutputs;
use crate::series::NodeCache;
use crate::types::{IndicatorKind, IndicatorOutput, MacdParams};

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
        IndicatorKind::SUPERTREND => supertrend_store(store, period, multiplier, nodes).into_outputs(),
        IndicatorKind::KELTNER => keltner_store(store, period, multiplier, nodes).into_outputs(),
        IndicatorKind::STARC => starc_store(store, period, multiplier, nodes).into_outputs(),
        IndicatorKind::WMA => wma_store(store, period, nodes).into_outputs(),
        IndicatorKind::HMA => hma_store(store, period, nodes).into_outputs(),
        IndicatorKind::LINEAR_REGRESSION => linear_regression_store(store, period, nodes).into_outputs(),
        IndicatorKind::LINEAR_REG_FORECAST => linear_reg_forecast_store(store, period, nodes).into_outputs(),
        IndicatorKind::LINEAR_REG_INTERCEPT => linear_reg_intercept_store(store, period, nodes).into_outputs(),
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
        IndicatorKind::ICHIMOKU => ichimoku_store(store, tenkan_period, kijun_period, senkou_b_period, nodes)
            .into_outputs(),
        IndicatorKind::PIVOT_POINTS => pivot_points_store(store, nodes).into_outputs(),
        IndicatorKind::AROON => aroon_store(store, period, nodes).into_outputs(),
        IndicatorKind::ULTIMATE_OSCILLATOR => {
            ultimate_oscillator_store(store, period, stoch_period, smooth, nodes).into_outputs()
        }
        IndicatorKind::CHAIKIN_VOLATILITY => chaikin_volatility_store(store, period, nodes).into_outputs(),
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
        IndicatorKind::ATR_BANDS => atr_bands_store(store, period, multiplier, nodes).into_outputs(),
        IndicatorKind::HIGH_LOW_BANDS => high_low_bands_store(store, period, nodes).into_outputs(),
        IndicatorKind::FRACTAL_CHAOS_BANDS => fractal_chaos_bands_store(store, nodes).into_outputs(),
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
        IndicatorKind::TIME_SERIES_FORECAST => linear_reg_forecast_store(store, period, nodes).into_outputs(),
        IndicatorKind::VALUATION_LINES => valuation_lines_store(store, period, multiplier, nodes).into_outputs(),
        IndicatorKind::BETA => beta_store(store, period, nodes).into_outputs(),
        IndicatorKind::CORRELATION_COEFFICIENT => {
            correlation_coefficient_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::PERFORMANCE_INDEX => performance_index_store(store, nodes).into_outputs(),
        IndicatorKind::PRICE_RELATIVE => price_relative_store(store, period, nodes).into_outputs(),
        IndicatorKind::AWESOME_OSCILLATOR => awesome_oscillator_store(store, nodes).into_outputs(),
        IndicatorKind::BOLLINGER_PCT_B => bollinger_pct_b_store(store, period, multiplier, nodes).into_outputs(),
        IndicatorKind::CENTER_OF_GRAVITY => center_of_gravity_store(store, period, nodes).into_outputs(),
        IndicatorKind::CHANDE_FORECAST => chande_forecast_store(store, period, nodes).into_outputs(),
        IndicatorKind::CHANDE_MOMENTUM => chande_momentum_store(store, period, nodes).into_outputs(),
        IndicatorKind::COPPOCK_CURVE => coppock_curve_store(store, nodes).into_outputs(),
        IndicatorKind::DISPARITY_INDEX => disparity_index_store(store, period, nodes).into_outputs(),
        IndicatorKind::EASE_OF_MOVEMENT => ease_of_movement_store(store, period, nodes).into_outputs(),
        IndicatorKind::EHLER_FISHER => ehler_fisher_store(store, period, nodes).into_outputs(),
        IndicatorKind::ELDER_RAY => elder_ray_store(store, period, nodes).into_outputs(),
        IndicatorKind::FRACTAL_CHAOS_OSCILLATOR => fractal_chaos_oscillator_store(store, nodes).into_outputs(),
        IndicatorKind::GATOR_OSCILLATOR => gator_oscillator_store(store, nodes).into_outputs(),
        IndicatorKind::INTRADAY_MOMENTUM => intraday_momentum_store(store, period, nodes).into_outputs(),
        IndicatorKind::LINEAR_REG_SLOPE => linear_reg_slope_store(store, period, nodes).into_outputs(),
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
        IndicatorKind::RAINBOW_OSCILLATOR => rainbow_oscillator_store(store, period, nodes).into_outputs(),
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
        IndicatorKind::TREND_INTENSITY => trend_intensity_store(store, period, nodes).into_outputs(),
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
        IndicatorKind::MARKET_FACILITATION => market_facilitation_store(store, nodes).into_outputs(),
        IndicatorKind::NEGATIVE_VOLUME_INDEX => negative_volume_index_store(store, nodes).into_outputs(),
        IndicatorKind::POSITIVE_VOLUME_INDEX => positive_volume_index_store(store, nodes).into_outputs(),
        IndicatorKind::PRICE_VOLUME_TREND => price_volume_trend_store(store, nodes).into_outputs(),
        IndicatorKind::TRADE_VOLUME_INDEX => trade_volume_index_store(store, nodes).into_outputs(),
        IndicatorKind::TWIGGS_MONEY_FLOW => twiggs_money_flow_store(store, period, nodes).into_outputs(),
        IndicatorKind::PROJECTED_AGGREGATE_VOLUME => {
            projected_aggregate_volume_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::PROJECTED_VOLUME_AT_TIME => {
            projected_volume_at_time_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::HISTORICAL_VOLATILITY => historical_volatility_store(store, period, nodes).into_outputs(),
        IndicatorKind::LINEAR_REG_R2 => linear_reg_r2_store(store, period, nodes).into_outputs(),
        IndicatorKind::PRIME_NUMBER_OSCILLATOR => prime_number_oscillator_store(store, nodes).into_outputs(),
        IndicatorKind::RANDOM_WALK_INDEX => random_walk_index_store(store, period, nodes).into_outputs(),
        IndicatorKind::DARVAS_BOX => darvas_box_store(store, nodes).into_outputs(),
        IndicatorKind::VOLUME_PROFILE => volume_profile_store(store, period, nodes).into_outputs(),
        IndicatorKind::CHOPPINESS_INDEX => choppiness_index_store(store, period, nodes).into_outputs(),
        IndicatorKind::ELDER_IMPULSE => elder_impulse_store(store, period, nodes).into_outputs(),
        IndicatorKind::GONOGO_TREND => gonogo_trend_store(store, period, nodes).into_outputs(),
        IndicatorKind::PSYCHOLOGICAL_LINE => psychological_line_store(store, period, nodes).into_outputs(),
        IndicatorKind::QSTICK => qstick_store(store, period, nodes).into_outputs(),
        IndicatorKind::SHINOHARA_INTENSITY => shinohara_intensity_store(store, period, nodes).into_outputs(),
        IndicatorKind::ULCER_INDEX => ulcer_index_store(store, period, nodes).into_outputs(),
        IndicatorKind::VERTICAL_HORIZONTAL_FILTER => {
            vertical_horizontal_filter_store(store, period, nodes).into_outputs()
        }
        IndicatorKind::VORTEX_INDICATOR => vortex_indicator_store(store, period, nodes).into_outputs(),
        IndicatorKind::ZIGZAG => zigzag_store(store, multiplier, nodes).into_outputs(),
        IndicatorKind::BOLLINGER_BANDWIDTH => {
            bollinger_bandwidth_store(store, period, multiplier, nodes).into_outputs()
        }
        IndicatorKind::DONCHIAN_WIDTH => donchian_width_store(store, period, nodes).into_outputs(),
        IndicatorKind::GOPALAKRISHNAN_RANGE => gopalakrishnan_range_store(store, period, nodes).into_outputs(),
        IndicatorKind::HIGH_MINUS_LOW => high_minus_low_store(store, nodes).into_outputs(),
        IndicatorKind::MASS_INDEX => mass_index_store(store, period, nodes).into_outputs(),
        IndicatorKind::RELATIVE_VOLATILITY => relative_volatility_store(store, period, nodes).into_outputs(),
        IndicatorKind::TRUE_RANGE => true_range_series_store(store, nodes).into_outputs(),
        IndicatorKind::VOLUME_CHART => volume_chart_store(store, nodes).into_outputs(),
        IndicatorKind::VOLUME_ROC => volume_roc_store(store, period, nodes).into_outputs(),
        IndicatorKind::VOLUME_UNDERLAY => volume_underlay_store(store, nodes).into_outputs(),
    }
}
