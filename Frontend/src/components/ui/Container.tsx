import type { HTMLAttributes } from "react";

type ContainerProps = HTMLAttributes<HTMLDivElement>;

export function Container({
  className = "",
  children,
  ...props
}: ContainerProps) {
  return (
    <div
      className={`mx-auto w-full max-w-355 px-5 sm:px-7 lg:px-10 ${className}`}
      {...props}
    >
      {children}
    </div>
  );
}
