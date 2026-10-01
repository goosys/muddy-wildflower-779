type IconProps = {
  name: "copy" | "check" | "refresh" | "arrow";
  className?: string;
};
export function Icon({ name, className }: IconProps) {
  const paths = {
    copy: "M9 9h11v11H9z M5 15H3V3h12v2",
    check: "m5 12 4 4L19 6",
    refresh:
      "M20 7v5h-5 M4 17v-5h5 M6.1 6.1a8 8 0 0 1 13.1 2.4 M4.8 15.5a8 8 0 0 0 13.1 2.4",
    arrow: "M7 17 17 7 M7 7h10v10",
  };
  return (
    <svg
      aria-hidden="true"
      className={className}
      width="20"
      height="20"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.7"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <path d={paths[name]} />
    </svg>
  );
}
