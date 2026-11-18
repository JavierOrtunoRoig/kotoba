type IconProps = {
  size?: number;
  className?: string;
};

function svgProps({ size = 20, className }: IconProps) {
  return {
    width: size,
    height: size,
    viewBox: "0 0 24 24",
    fill: "none",
    stroke: "currentColor",
    strokeWidth: 1.6,
    strokeLinecap: "round" as const,
    strokeLinejoin: "round" as const,
    className,
    "aria-hidden": true,
  };
}

export function IconPractice(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <rect x="4" y="5" width="16" height="14" rx="2.5" />
      <path d="M8 9h8M8 13h5" />
      <path d="M15.5 15.5l2.5 2.5" />
    </svg>
  );
}

export function IconChart(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M5 19V9M12 19V5M19 19v-7" />
      <path d="M3.5 19.5h17" />
    </svg>
  );
}

export function IconSettings(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M4 7h10M18 7h2M4 17h2M10 17h10" />
      <circle cx="16" cy="7" r="2.2" />
      <circle cx="8" cy="17" r="2.2" />
    </svg>
  );
}

export function IconClose(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M6 6l12 12M18 6L6 18" />
    </svg>
  );
}

export function IconSound(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M4 10v4h3l4 3V7L7 10H4z" />
      <path d="M15 9.5a4 4 0 010 5M17.5 7a7.5 7.5 0 010 10" />
    </svg>
  );
}

export function IconArrow(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M5 12h14M13 6l6 6-6 6" />
    </svg>
  );
}

export function IconImport(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M12 4v10M8 10l4 4 4-4" />
      <path d="M5 16v3h14v-3" />
    </svg>
  );
}

export function IconCheck(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M5 12.5l4.5 4.5L19 7" />
    </svg>
  );
}

export function IconRefresh(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M4.5 11a7.5 7.5 0 0112.8-5.3L20 8" />
      <path d="M20 4v4h-4" />
      <path d="M19.5 13a7.5 7.5 0 01-12.8 5.3L4 16" />
      <path d="M4 20v-4h4" />
    </svg>
  );
}

export function IconTrash(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M5 7h14M10 7V5h4v2M7 7l1 12h8l1-12" />
      <path d="M10.5 10.5v5M13.5 10.5v5" />
    </svg>
  );
}

export function IconFlame(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M12 3c1.8 3.2 5.5 5.2 5.5 9.2A5.5 5.5 0 0112 18a5.5 5.5 0 01-5.5-5.8C6.5 8.6 9.4 6.2 12 3z" />
      <path d="M12 18v-4.5" />
    </svg>
  );
}

export function IconTarget(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <circle cx="12" cy="12" r="8" />
      <circle cx="12" cy="12" r="4" />
      <circle cx="12" cy="12" r="1" />
    </svg>
  );
}

export function IconBook(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M5 5.5A2.5 2.5 0 017.5 3H19v16H7.5A2.5 2.5 0 005 21.5v-16z" />
      <path d="M5 18.5A2.5 2.5 0 017.5 16H19" />
    </svg>
  );
}

export function IconEye(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M3 12s3.5-6 9-6 9 6 9 6-3.5 6-9 6-9-6-9-6z" />
      <circle cx="12" cy="12" r="2.5" />
    </svg>
  );
}

export function IconSeed(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M12 21V11" />
      <path d="M12 14c-4 0-6-2.5-6-6 3.5 0 6 2 6 6z" />
      <path d="M12 11c0-3.8 2.4-6 6-6 0 3.4-2.2 6-6 6z" />
    </svg>
  );
}

export function IconArchive(props: IconProps) {
  return (
    <svg {...svgProps(props)}>
      <path d="M4 8h16v12H4z" />
      <path d="M3 5h18v3H3z" />
      <path d="M10 12h4" />
    </svg>
  );
}
