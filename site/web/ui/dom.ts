// Element builders. Every view builds DOM through h() (HTML) and s() (SVG); nothing assigns innerHTML except prose().

export type Child = Node | string | number | null | undefined | false | Child[];
export type Attrs = Record<string, string | number | boolean | undefined | null>;

function append(el: Element, children: Child[]) {
  const add = (child: Child) => {
    if (child === null || child === undefined || child === false) return;
    if (Array.isArray(child)) child.forEach(add);
    else el.append(child instanceof Node ? child : String(child));
  };
  children.forEach(add);
}

function setAttrs(el: Element, attrs: Attrs) {
  for (const [key, value] of Object.entries(attrs)) {
    if (value === undefined || value === null || value === false) continue;
    el.setAttribute(key, value === true ? "" : String(value));
  }
}

export function h(tag: string, attrs: Attrs = {}, ...children: Child[]): HTMLElement {
  const el = document.createElement(tag);
  setAttrs(el, attrs);
  append(el, children);
  return el;
}

export function s(tag: string, attrs: Attrs = {}, ...children: Child[]): SVGElement {
  const el = document.createElementNS("http://www.w3.org/2000/svg", tag);
  setAttrs(el, attrs);
  append(el, children);
  return el;
}

/** Items separated by `sep`, for inline lists of links or ids. */
export const join = (items: Child[], sep: Child = ", "): Child[] => items.flatMap((item, i) => (i === 0 ? [item] : [sep, item]));

export function countBy<T>(rows: T[], key: (row: T) => string): [string, number][] {
  const counts = new Map<string, number>();
  for (const row of rows) counts.set(key(row), (counts.get(key(row)) ?? 0) + 1);
  return [...counts];
}
