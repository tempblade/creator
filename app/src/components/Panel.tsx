import { FC, ReactNode } from "react";

const Panel: FC<{ title: string; children: ReactNode }> = ({
  title,
  children,
}) => {
  return (
    <div>
      <h3>{title}</h3>
      <div>{children}</div>
    </div>
  );
};

export default Panel;
