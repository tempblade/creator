import * as ScrollArea from "@radix-ui/react-scroll-area";
import { FC } from "react";

const ScrollBar: FC<{ orientation?: "horizontal" | "vertical" }> = ({
  orientation = "vertical",
}) => {
  return (
    <ScrollArea.Scrollbar
      className="flex select-none touch-none p-0.5 bg-neutral-accent transition-colors duration-[160ms] ease-out data-[orientation=vertical]:w-2.5 data-[orientation=horizontal]:flex-col data-[orientation=horizontal]:h-2.5"
      orientation={orientation}
    >
      <ScrollArea.Thumb className="flex-1 bg-main rounded-[10px] relative before:content-[''] before:absolute before:top-1/2 before:left-1/2 before:-translate-x-1/2 before:-translate-y-1/2 before:w-full before:h-full before:min-w-[44px] before:min-h-[44px]" />
    </ScrollArea.Scrollbar>
  );
};

export default ScrollBar;
