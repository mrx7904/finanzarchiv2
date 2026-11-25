// The icons of the app: inline SVG on a 20 px grid with a 1.6 stroke in the current text color, so nothing is loaded
// and an icon takes the color of its control. Drawn for this app, no icon set copied.
import type { ReactNode, SVGProps } from "react";

const paths = {
  home: <path d="M3.5 9 10 3.5 16.5 9v7a1 1 0 0 1-1 1h-3v-5h-5v5h-3a1 1 0 0 1-1-1z" />,
  inbox: (
    <>
      <path d="M3 11h4l1 2h4l1-2h4" />
      <path d="M5 4h10l2 7v5a1 1 0 0 1-1 1H4a1 1 0 0 1-1-1v-5z" />
    </>
  ),
  document: (
    <>
      <path d="M5 2.5h6.5L15 6v11a.5.5 0 0 1-.5.5h-9A.5.5 0 0 1 5 17z" />
      <path d="M11.5 2.5V6H15M7.5 10h5M7.5 13h5" />
    </>
  ),
  receipt: (
    <>
      <path d="M5 2.5h10v15l-2-1.2-1.6 1.2-1.4-1.2-1.4 1.2L7 16.3l-2 1.2z" />
      <path d="M7.5 7h5M7.5 10h5M7.5 13h3" />
    </>
  ),
  card: (
    <>
      <rect x="2.5" y="4.5" width="15" height="11" rx="1.5" />
      <path d="M2.5 8h15M5.5 12.5h3" />
    </>
  ),
  scale: (
    <>
      <path d="M10 3v14M6 17h8M4 6h12" />
      <path d="M4 6 2 11a2.5 2.5 0 0 0 4 0zM16 6l-2 5a2.5 2.5 0 0 0 4 0z" />
    </>
  ),
  gear: (
    <>
      <circle cx="10" cy="10" r="2.5" />
      <path d="M10 2.5v2M10 15.5v2M2.5 10h2M15.5 10h2M4.7 4.7l1.4 1.4M13.9 13.9l1.4 1.4M4.7 15.3l1.4-1.4M13.9 6.1l1.4-1.4" />
    </>
  ),
  search: (
    <>
      <circle cx="9" cy="9" r="5.5" />
      <path d="m13 13 4 4" />
    </>
  ),
  plus: <path d="M10 4v12M4 10h12" />,
  chevronDown: <path d="m6 8 4 4 4-4" />,
  upDown: <path d="m7 8 3-3 3 3M7 12l3 3 3-3" />,
  check: <path d="m4.5 10.5 3.5 3.5 7.5-8" />,
  close: <path d="m5.5 5.5 9 9M14.5 5.5l-9 9" />,
  trash: (
    <>
      <path d="M3.5 5.5h13M8 5.5V3.5h4v2" />
      <path d="M5 5.5 6 17h8l1-11.5" />
    </>
  ),
  more: (
    <>
      <circle cx="5" cy="10" r="0.9" fill="currentColor" />
      <circle cx="10" cy="10" r="0.9" fill="currentColor" />
      <circle cx="15" cy="10" r="0.9" fill="currentColor" />
    </>
  ),
  folder: <path d="M2.5 5.5a1 1 0 0 1 1-1h4l1.5 2h7.5a1 1 0 0 1 1 1v8a1 1 0 0 1-1 1h-13a1 1 0 0 1-1-1z" />,
  upload: <path d="M10 13V3.5M6 7.5l4-4 4 4M3.5 13v3a1 1 0 0 0 1 1h11a1 1 0 0 0 1-1v-3" />,
  mail: (
    <>
      <rect x="2.5" y="4.5" width="15" height="11" rx="1.5" />
      <path d="m3 5.5 7 5.5 7-5.5" />
    </>
  ),
} satisfies Record<string, ReactNode>;

export type IconName = keyof typeof paths;

export function Icon({ name, size = 16, className, ...props }: { name: IconName; size?: number } & SVGProps<SVGSVGElement>) {
  return (
    <svg
      className={className ? `shrink-0 ${className}` : "shrink-0"}
      width={size}
      height={size}
      viewBox="0 0 20 20"
      fill="none"
      stroke="currentColor"
      strokeWidth={1.6}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden
      focusable={false}
      {...props}
    >
      {paths[name]}
    </svg>
  );
}
