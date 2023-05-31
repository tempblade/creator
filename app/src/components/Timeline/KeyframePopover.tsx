import { PopoverClose } from "components/Popover";
import { KeyframeProperties } from "components/Properties/Values";
import { Keyframe } from "primitives/Keyframe";
import { FC } from "react";
import { z } from "zod";

const KeyframePopover: FC<{
  keyframe: z.input<typeof Keyframe>;
  onUpdate: (k: z.input<typeof Keyframe>) => void;
  onClose: () => void;
}> = ({ keyframe, onUpdate, onClose }) => {
  return (
    <div>
      <KeyframeProperties entity={keyframe} onUpdate={onUpdate} />
      <PopoverClose onClick={onClose} />
    </div>
  );
};

export default KeyframePopover;
