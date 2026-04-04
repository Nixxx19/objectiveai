namespace ObjectiveAI.Agent.Completions.Message;

public partial class RichContent
{
    public void Push(RichContent other)
    {
        if (Text != null && other.Text != null)
        {
            // text + text → concatenate
            Text += other.Text;
        }
        else if (Text != null && other.Parts != null)
        {
            // text + parts → convert text to part, extend with other.Parts
            var parts = new List<RichContentPart>
            {
                new RichContentPart
                {
                    Text = new RichContentPartText { Type = "text", Text = Text }
                }
            };
            parts.AddRange(other.Parts);
            Text = null;
            Parts = parts;
        }
        else if (Parts != null && other.Text != null)
        {
            // parts + text → append text as new part (if non-empty)
            if (!string.IsNullOrEmpty(other.Text))
            {
                Parts.Add(new RichContentPart
                {
                    Text = new RichContentPartText { Type = "text", Text = other.Text }
                });
            }
        }
        else if (Parts != null && other.Parts != null)
        {
            // parts + parts → extend
            Parts.AddRange(other.Parts);
        }
    }
}
