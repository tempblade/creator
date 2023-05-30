import { FC, ReactNode } from "react";
import * as ToggleGroupComponents from "@radix-ui/react-toggle-group";

const ToggleGroupItem: FC<{
  children: ReactNode;
  selected: boolean;
  onClick?: () => void;
}> = ({ children, selected, onClick }) => {
  return (
    <ToggleGroupComponents.Item
      data-selected={selected}
      onClick={onClick}
      className="hover:bg-indigo-400 text-white data-[selected=true]:bg-indigo-600 
      data-[selected=true]:text-indigo-200 flex h-6 w-6
      items-center justify-center bg-slate-900 text-sm leading-4 
      first:rounded-l last:rounded-r focus:z-10 focus:shadow-[0_0_0_2px] focus:shadow-black 
      focus:outline-none"
      value="left"
      aria-label="Left aligned"
    >
      {children}
    </ToggleGroupComponents.Item>
  );
};

const ToggleGroup: FC<{ children: ReactNode }> = ({ children }) => (
  <ToggleGroupComponents.Root
    className="inline-flex bg-slate-800 rounded shadow-[0_2px_10px] shadow-black space-x-px"
    type="single"
    defaultValue="center"
    aria-label="Text alignment"
  >
    {children}
  </ToggleGroupComponents.Root>
);

export { ToggleGroup, ToggleGroupItem };
