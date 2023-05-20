import {
  BoxIcon,
  CircleIcon,
  CursorArrowIcon,
  MixIcon,
  Pencil1Icon,
  Pencil2Icon,
  SymbolIcon,
  TextIcon,
} from "@radix-ui/react-icons";
import * as Toolbar from "@radix-ui/react-toolbar";
import { FC, ReactNode } from "react";

const ToolBarButton: FC<{ children: ReactNode }> = ({ children }) => {
  return (
    <Toolbar.Button
      className="text-white p-[10px] bg-gray-900 flex-shrink-0 flex-grow-0 
  basis-auto w-[40px] h-[40px] rounded inline-flex text-[13px] leading-none 
  items-center justify-center outline-none hover:bg-indigo-900 
  transition-colors
  focus:relative focus:shadow-[0_0_0_2px] focus:shadow-indigo"
    >
      {children}
    </Toolbar.Button>
  );
};

const ToolBar = () => {
  return (
    <Toolbar.Root
      className="bg-gray-800 flex flex-col gap-1 p-1 h-full"
      orientation="vertical"
    >
      <ToolBarButton>
        <CursorArrowIcon width="100%" height="100%" />
      </ToolBarButton>
      <Toolbar.Separator />
      <ToolBarButton>
        <BoxIcon width="100%" height="100%" />
      </ToolBarButton>
      <ToolBarButton>
        <CircleIcon width="100%" height="100%" />
      </ToolBarButton>
      <ToolBarButton>
        <Pencil1Icon width="100%" height="100%" />
      </ToolBarButton>
      <ToolBarButton>
        <MixIcon width="100%" height="100%" />
      </ToolBarButton>
      <Toolbar.Separator />
      <ToolBarButton>
        <TextIcon width="100%" height="100%" />
      </ToolBarButton>
    </Toolbar.Root>
  );
};

export default ToolBar;
