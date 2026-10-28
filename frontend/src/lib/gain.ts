const MINIMUM_GAIN_DB = -100;

export function linearGainToDecibels(value: number) {
    if (value <= 0) {
        return MINIMUM_GAIN_DB;
    }

    return Math.max(MINIMUM_GAIN_DB, Math.round(20 * Math.log10(value) * 10) / 10);
}

export function decibelsToLinearGain(value: number) {
    return 10 ** (value / 20);
}
