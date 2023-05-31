import { FC, ReactNode } from "react";
import * as ToggleGroupComponents from "@radix-ui/react-toggle-group";
import { motion } from "framer-motion";

const ToggleGroupItem: FC<{
  children: ReactNode;
  selected: boolean;
  onClick?: () => void;
}> = ({ children, selected, onClick }) => {
  return (
    <ToggleGroupComponents.Item
      data-selected={selected}
      asChild
      onClick={onClick}
      className="hover:bg-indigo-600 text-white data-[selected=true]:bg-indigo-700 
      data-[selected=true]:text-indigo-200 flex h-6 w-6
      items-center justify-center bg-slate-900 text-sm leading-4 
      first:rounded-l last:rounded-r focus:z-10 focus:shadow-[0_0_0_2px] focus:shadow-black 
      focus:outline-none transition-colors"
      value="left"
      aria-label="Left aligned"
    >
      <motion.button animate={{ scale: 1 }} whileTap={{ scale: 0.9 }}>
        {children}
      </motion.button>
    </ToggleGroupComponents.Item>
  );
};

const ToggleGroup: FC<{ children: ReactNode }> = ({ children }) => (
  <ToggleGroupComponents.Root
    className="inline-flex my-auto bg-slate-800 h-fit rounded shadow-[0_2px_10px] shadow-black space-x-px"
    type="single"
    defaultValue="center"
    aria-label="Text alignment"
  >
    {children}
  </ToggleGroupComponents.Root>
);

export { ToggleGroup, ToggleGroupItem };
