import {
  BoxIcon,
  CircleIcon,
  CursorArrowIcon,
  FontStyleIcon,
  MixIcon,
  Pencil1Icon,
  Pencil2Icon,
  SymbolIcon,
  TextIcon,
} from "@radix-ui/react-icons";
import * as Toolbar from "@radix-ui/react-toolbar";
import { motion } from "framer-motion";
import { FC, ReactNode, useMemo, useState } from "react";
import { EntitiesService } from "services/entities.service";

const ToolBarButton: FC<{ children: ReactNode; onClick?: () => void }> = ({
  children,
  onClick,
}) => {
  const [didHover, setDidHover] = useState(false);

  return (
    <Toolbar.Button
      onClick={onClick}
      onMouseOver={() => !didHover && setDidHover(true)}
      asChild
      className="text-white p-[10px] bg-gray-900 flex-shrink-0 flex-grow-0 
  basis-auto w-[40px] h-[40px] rounded inline-flex text-[13px] leading-none 
  items-center justify-center outline-none hover:bg-indigo-900 
  transition-colors
  focus:relative focus:shadow-[0_0_0_2px] focus:shadow-indigo"
    >
      <motion.button
        animate={didHover ? "enter" : undefined}
        whileTap="press"
        variants={{
          enter: { scale: 1 },
          from: { scale: 0 },
          press: { scale: 0.9 },
        }}
      >
        {children}
      </motion.button>
    </Toolbar.Button>
  );
};

const ToolBar = () => {
  const entitiesService = useMemo(() => new EntitiesService(), []);

  return (
    <Toolbar.Root
      asChild
      className="bg-gray-800 flex flex-col gap-1 p-1 h-full"
      orientation="vertical"
    >
      <motion.div
        animate="enter"
        initial="from"
        transition={{ staggerChildren: 0.1 }}
        variants={{ enter: {}, from: {} }}
      >
        <ToolBarButton>
          <CursorArrowIcon width="100%" height="100%" />
        </ToolBarButton>
        <Toolbar.Separator />
        <ToolBarButton onClick={() => entitiesService.createRect()}>
          <BoxIcon width="100%" height="100%" />
        </ToolBarButton>
        <ToolBarButton onClick={() => entitiesService.createEllipse()}>
          <CircleIcon width="100%" height="100%" />
        </ToolBarButton>
        <ToolBarButton>
          <Pencil1Icon width="100%" height="100%" />
        </ToolBarButton>
        <ToolBarButton>
          <MixIcon width="100%" height="100%" />
        </ToolBarButton>
        <Toolbar.Separator />
        <ToolBarButton onClick={() => entitiesService.createText()}>
          <TextIcon width="100%" height="100%" />
        </ToolBarButton>
        <ToolBarButton onClick={() => entitiesService.createStaggeredText()}>
          <FontStyleIcon width="100%" height="100%" />
        </ToolBarButton>
      </motion.div>
    </Toolbar.Root>
  );
};

export default ToolBar;
