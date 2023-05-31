import * as React from "react";
import * as PopoverPrimitive from "@radix-ui/react-popover";
import { cn } from "utils";
import { Cross2Icon } from "@radix-ui/react-icons";

const Popover = PopoverPrimitive.Root;

const PopoverTrigger = PopoverPrimitive.Trigger;

const PopoverArrow = PopoverPrimitive.Arrow;

const PopoverClose: React.FC<{ onClick?: () => void }> = ({ onClick }) => (
  <PopoverPrimitive.Close
    onClick={onClick}
    className="rounded-full h-[25px] w-[25px] inline-flex items-center justify-center text-white absolute top-[5px] right-[5px] hover:bg-indigo-600 focus:shadow-[0_0_0_2px] focus:shadow-indigo-500 outline-none cursor-default"
    aria-label="Close"
  >
    <Cross2Icon />
  </PopoverPrimitive.Close>
);

const PopoverContent = React.forwardRef<
  React.ElementRef<typeof PopoverPrimitive.Content>,
  React.ComponentPropsWithoutRef<typeof PopoverPrimitive.Content>
>(({ className, align = "center", sideOffset = 4, ...props }, ref) => (
  <PopoverPrimitive.Portal>
    <PopoverPrimitive.Content
      ref={ref}
      align={align}
      sideOffset={sideOffset}
      className={cn(
        "z-50 w-72 rounded-md border bg-popover p-4 text-popover-foreground shadow-md outline-none animate-in data-[side=bottom]:slide-in-from-top-2 data-[side=left]:slide-in-from-right-2 data-[side=right]:slide-in-from-left-2 data-[side=top]:slide-in-from-bottom-2",
        className
      )}
      {...props}
    />
  </PopoverPrimitive.Portal>
));
PopoverContent.displayName = PopoverPrimitive.Content.displayName;

export { Popover, PopoverTrigger, PopoverClose, PopoverContent, PopoverArrow };
