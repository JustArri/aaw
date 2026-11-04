const blackKeyClasses = new Set([1, 3, 6, 8, 10]);

export const keyboardAutoViewStorageKey = 'aaw-keyboard-auto-view';
export const whiteKeyWidthRem = 2;
export const blackKeyWidthRem = 1.25;
export const blackKeyWidthRatio = blackKeyWidthRem / whiteKeyWidthRem;
export const keyboardAutoViewPaddingKeys = 2;
export const keyboardActivityTimerIntervalMs = 250;

export type KeyboardKey = {
    note: number;
    whiteIndex: number;
    black: boolean;
    left: number;
};

export function buildKeyboard() {
    const keys: KeyboardKey[] = [];
    let whiteCount = 0;
    for (let note = 0; note < 128; note += 1) {
        const black = blackKeyClasses.has(note % 12);
        if (black) {
            keys.push({
                note,
                whiteIndex: -1,
                black,
                left: whiteCount * whiteKeyWidthRem - blackKeyWidthRem / 2
            });
        } else {
            keys.push({
                note,
                whiteIndex: whiteCount,
                black,
                left: whiteCount * whiteKeyWidthRem
            });
            whiteCount += 1;
        }
    }
    return { keys, whiteCount };
}
