import { useState } from "react";
import { Button } from "./ui/button";
import { Input } from "./ui/input";
import { Textarea } from "./ui/textarea";
import { Card, CardDescription, CardHeader, CardTitle } from "./ui/card";
import { Project, CreateProjectInput } from "../types";
import { Plus, Folder } from "lucide-react";

interface SidebarProps {
  projects: Project[];
  selectedProject: Project | null;
  onSelectProject: (project: Project) => void;
  onCreateProject: (input: CreateProjectInput) => void;
}

export function Sidebar({
  projects,
  selectedProject,
  onSelectProject,
  onCreateProject,
}: SidebarProps) {
  const [showCreateForm, setShowCreateForm] = useState(false);
  const [formData, setFormData] = useState<CreateProjectInput>({
    name: "",
    description: "",
    industry: "",
    target_audience: "",
  });

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();
    onCreateProject(formData);
    setFormData({
      name: "",
      description: "",
      industry: "",
      target_audience: "",
    });
    setShowCreateForm(false);
  };

  return (
    <div className="w-80 border-r bg-gray-50 flex flex-col h-screen">
      <div className="p-4 border-b">
        <h2 className="text-lg font-semibold mb-3">Projects</h2>
        <Button
          onClick={() => setShowCreateForm(!showCreateForm)}
          className="w-full"
          variant="outline"
        >
          <Plus className="w-4 h-4 mr-2" />
          New Project
        </Button>
      </div>

      {showCreateForm && (
        <div className="p-4 border-b bg-white">
          <form onSubmit={handleSubmit} className="space-y-3">
            <Input
              placeholder="Project name"
              value={formData.name}
              onChange={(e) =>
                setFormData({ ...formData, name: e.target.value })
              }
              required
            />
            <Textarea
              placeholder="Description"
              value={formData.description}
              onChange={(e) =>
                setFormData({ ...formData, description: e.target.value })
              }
              required
              rows={3}
            />
            <Input
              placeholder="Industry (optional)"
              value={formData.industry}
              onChange={(e) =>
                setFormData({ ...formData, industry: e.target.value })
              }
            />
            <Input
              placeholder="Target audience (optional)"
              value={formData.target_audience}
              onChange={(e) =>
                setFormData({
                  ...formData,
                  target_audience: e.target.value,
                })
              }
            />
            <div className="flex gap-2">
              <Button type="submit" size="sm">
                Create
              </Button>
              <Button
                type="button"
                size="sm"
                variant="outline"
                onClick={() => setShowCreateForm(false)}
              >
                Cancel
              </Button>
            </div>
          </form>
        </div>
      )}

      <div className="flex-1 overflow-y-auto p-4 space-y-2">
        {projects.length === 0 ? (
          <div className="text-center text-gray-500 py-8">
            <Folder className="w-12 h-12 mx-auto mb-2 opacity-50" />
            <p className="text-sm">No projects yet</p>
          </div>
        ) : (
          projects.map((project) => (
            <Card
              key={project.id}
              className={`cursor-pointer transition-colors hover:bg-gray-100 ${
                selectedProject?.id === project.id ? "bg-gray-100" : ""
              }`}
              onClick={() => onSelectProject(project)}
            >
              <CardHeader className="p-4">
                <CardTitle className="text-base">{project.name}</CardTitle>
                <CardDescription className="text-xs line-clamp-2">
                  {project.description}
                </CardDescription>
              </CardHeader>
            </Card>
          ))
        )}
      </div>
    </div>
  );
}
