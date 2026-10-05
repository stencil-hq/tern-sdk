/** The `css` template tag. */

/**
 * A stylesheet as a template literal: returns the text as written (CSS
 * escapes such as `\2192` kept), so editors highlight it as CSS.
 */
export function css(strings: TemplateStringsArray, ...values: readonly unknown[]): string {
   return String.raw(strings, ...values);
}
