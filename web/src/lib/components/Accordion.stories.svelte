<script module lang="ts">
  import { defineMeta } from '@storybook/addon-svelte-csf';
  import Accordion from './Accordion.svelte';

  const { Story } = defineMeta({
    title: 'Components/Accordion',
    component: Accordion,
  });
</script>

<Story name="Default">
  <Accordion title="What is brutalism design?">
    Brutalism in web design is characterized by bold typography, thick borders,
    high contrast colors, and a raw, unapologetic aesthetic that emphasizes
    functionality over ornamentation.
  </Accordion>
</Story>

<Story name="Open">
  <Accordion title="This accordion is open by default" open={true}>
    The <code>open</code> prop controls whether the accordion starts in an open state.
    Users can still toggle it closed and open again.
  </Accordion>
</Story>

<Story name="Disabled">
  <Accordion title="This accordion is disabled" disabled={true}>
    This content cannot be accessed when the accordion is disabled.
  </Accordion>
</Story>

<Story name="Multiple Accordions">
  <div class="flex flex-col gap-4">
    <Accordion title="Section 1: Introduction">
      This is the first section with some introductory content. You can put any
      content here including text, images, or other components.
    </Accordion>
    <Accordion title="Section 2: Details" open={true}>
      This section is open by default and contains more detailed information.
      Notice how each accordion is independent and maintains its own state.
    </Accordion>
    <Accordion title="Section 3: Advanced Topics">
      The third section contains advanced information. The brutalist styling
      with thick borders and box shadows makes each section clearly defined.
    </Accordion>
    <Accordion title="Section 4: Disabled Example" disabled={true}>
      This section is disabled and cannot be opened.
    </Accordion>
  </div>
</Story>

<Story name="With Rich Content">
  <Accordion title="FAQ: How do I use this component?">
    <div class="flex flex-col gap-3">
      <p><strong>Q: Can I nest other components inside?</strong></p>
      <p>A: Yes! You can include any content in the accordion body.</p>
      <div class="p-4 border-3 border-black bg-yellow-100">
        <p class="font-bold">Example nested content</p>
        <ul class="list-disc list-inside mt-2">
          <li>Lists work great</li>
          <li>So do styled boxes</li>
          <li>And any other HTML elements</li>
        </ul>
      </div>
    </div>
  </Accordion>
</Story>

<Story name="Long Content">
  <Accordion title="Terms and Conditions">
    <div class="flex flex-col gap-4">
      <p>
        Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod
        tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim
        veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea
        commodo consequat.
      </p>
      <p>
        Duis aute irure dolor in reprehenderit in voluptate velit esse cillum
        dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non
        proident, sunt in culpa qui officia deserunt mollit anim id est laborum.
      </p>
      <p>
        Sed ut perspiciatis unde omnis iste natus error sit voluptatem
        accusantium doloremque laudantium, totam rem aperiam, eaque ipsa quae ab
        illo inventore veritatis et quasi architecto beatae vitae dicta sunt
        explicabo.
      </p>
    </div>
  </Accordion>
</Story>

<Story name="Interactive Demo">
  <div class="flex flex-col gap-4">
    <h3 class="text-xl font-bold">Click to toggle accordions:</h3>
    <Accordion title="🎨 Design System">
      Our brutalist design system uses bold borders, sharp corners, and high
      contrast to create a distinctive visual language.
    </Accordion>
    <Accordion title="🔧 Configuration Options">
      Each accordion supports title, open state, and disabled state props for
      maximum flexibility.
    </Accordion>
    <Accordion title="📚 Documentation">
      Check out the other stories to see different use cases and configurations.
    </Accordion>
  </div>
</Story>
